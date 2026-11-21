//! Wadah DEK (`data_key`, DD A19.7, ADR-013 §3) dan cache DEK berbatas waktu (ADR-013 konsekuensi,
//! KEPUTUSAN-79, coding-standards §5.5).
//!
//! Alur:
//! - [`Keyring::create_data_key`]: DEK baru dari KEK → baris [`NewDataKey`] untuk di-`INSERT` pemanggil
//!   di transaksi use case yang sama dengan baris bisnisnya, dan [`DataKey`] untuk mengenkripsi.
//! - [`Keyring::open_data_key`]: dari baris [`DataKeyRecord`] yang dibaca pemanggil. Urutan
//!   pemeriksaan: DEK dimusnahkan (`shredded_at`/`wrapped_dek` kosong) → ditolak dan cache dibuang;
//!   wadah (`container_type` + `container_id`) wajib sama dengan yang diharapkan; baru kemudian
//!   cache atau KEK. Pemeriksaan wadah tetap di pustaka karena `consultation_chat` dan
//!   `consultation_clinical` berbagi `container_id` (DD §0.2).
//! - [`DataKey::seal`]/[`DataKey::open`] menolak konteks kolom yang wadahnya bukan wadah DEK ini.
//!
//! Penyimpanan baris `data_key` ada di modul `store` ([`crate::insert_data_key`], [`crate::load_data_key`],
//! F1-T60). [`DataKeyring`] membungkus [`Keyring`] sebagai objek trait (future ber-`Box`), supaya service
//! modul bisnis tidak perlu generik atas jenis KEK.

use crate::context::{ContainerType, EncryptionContext, System};
use crate::envelope::{AadV1, DataEncryptionKey, SecretBytes};
use crate::error::CryptoError;
use crate::transit::KeyEncryptionKey;
use foundation::{Clock, Sensitive, Uuid, new_id};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use time::{Duration, OffsetDateTime};

/// Wadah data: jenis dan ID entitas pemiliknya.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Container {
    /// Jenis wadah (`data_key.container_type`).
    pub container_type: ContainerType,
    /// ID entitas pemilik wadah (`data_key.container_id`).
    pub container_id: Uuid,
}

impl Container {
    /// Membuat penanda wadah.
    pub const fn new(container_type: ContainerType, container_id: Uuid) -> Self {
        Self {
            container_type,
            container_id,
        }
    }
}

/// Baris `data_key` baru untuk di-`INSERT` pemanggil (kolom migrasi F1-T17). `retention_*` boleh diisi
/// pemanggil sebelum `INSERT` bila sudah diketahui (DD A19.7, A19.10).
pub struct NewDataKey {
    /// `id` (UUIDv7, ADR-002).
    pub id: Uuid,
    /// `container_type`.
    pub container_type: ContainerType,
    /// `container_id`.
    pub container_id: Uuid,
    /// `wrapped_dek`: ciphertext Transit sebagai byte UTF-8 (opaque).
    pub wrapped_dek: Sensitive<Vec<u8>>,
    /// `kek_ref`.
    pub kek_ref: String,
    /// `retention_category`.
    pub retention_category: Option<String>,
    /// `retention_anchor_at`.
    pub retention_anchor_at: Option<OffsetDateTime>,
}

impl fmt::Debug for NewDataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewDataKey")
            .field("id", &self.id)
            .field("container_type", &self.container_type)
            .field("container_id", &self.container_id)
            .field("wrapped_dek", &self.wrapped_dek)
            .field("kek_ref", &self.kek_ref)
            .finish_non_exhaustive()
    }
}

/// Baris `data_key` yang dibaca pemanggil (kolom yang dibutuhkan untuk membuka DEK).
pub struct DataKeyRecord {
    /// `id`.
    pub id: Uuid,
    /// `container_type` apa adanya dari database.
    pub container_type: String,
    /// `container_id`.
    pub container_id: Uuid,
    /// `wrapped_dek`; `None` setelah dimusnahkan.
    pub wrapped_dek: Option<Sensitive<Vec<u8>>>,
    /// `kek_ref`.
    pub kek_ref: String,
    /// `shredded_at`.
    pub shredded_at: Option<OffsetDateTime>,
}

impl fmt::Debug for DataKeyRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataKeyRecord")
            .field("id", &self.id)
            .field("container_type", &self.container_type)
            .field("container_id", &self.container_id)
            .field("wrapped_dek", &self.wrapped_dek)
            .field("kek_ref", &self.kek_ref)
            .field("shredded_at", &self.shredded_at)
            .finish()
    }
}

/// DEK terbuka untuk satu wadah. Murah di-clone (DEK dibagi lewat `Arc`).
#[derive(Clone)]
pub struct DataKey {
    container: Container,
    system: System,
    dek: Arc<DataEncryptionKey>,
}

impl DataKey {
    /// ID baris `data_key` (untuk kolom `data_key_id` di baris bisnis).
    pub fn id(&self) -> Uuid {
        self.dek.data_key_id()
    }

    /// Wadah DEK ini.
    pub fn container(&self) -> Container {
        self.container
    }

    fn aad(&self, context: EncryptionContext, row_id: Uuid) -> Result<AadV1, CryptoError> {
        if context.container_type() != self.container.container_type {
            return Err(CryptoError::ContainerMismatch);
        }
        Ok(AadV1 {
            context,
            system: self.system,
            row_id,
        })
    }

    /// Mengenkripsi nilai kolom `context` milik baris `row_id`.
    pub fn seal(
        &self,
        context: EncryptionContext,
        row_id: Uuid,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let aad = self.aad(context, row_id)?;
        self.dek.seal(&aad, plaintext)
    }

    /// Membuka nilai kolom `context` milik baris `row_id`.
    pub fn open(
        &self,
        context: EncryptionContext,
        row_id: Uuid,
        envelope: &[u8],
    ) -> Result<SecretBytes, CryptoError> {
        let aad = self.aad(context, row_id)?;
        self.dek.open(&aad, envelope)
    }
}

impl fmt::Debug for DataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataKey")
            .field("id", &self.id())
            .field("container", &self.container)
            .field("system", &self.system)
            .finish_non_exhaustive()
    }
}

/// Konfigurasi keyring. Nilai TTL dan kapasitas production belum ditetapkan dokumen mana pun,
/// sehingga wajib diberikan eksplisit (tanpa nilai bawaan).
#[derive(Debug, Clone, Copy)]
pub struct KeyringConfig {
    /// Sistem pemilik DEK (AAD bidang 3).
    pub system: System,
    /// Umur maksimum DEK di cache memori.
    pub cache_ttl: Duration,
    /// Jumlah maksimum DEK di cache.
    pub cache_capacity: usize,
}

struct CacheEntry {
    container: Container,
    dek: Arc<DataEncryptionKey>,
    expires_at: OffsetDateTime,
}

/// Pengelola DEK: KEK + cache DEK berbatas waktu di memori proses.
pub struct Keyring<K> {
    kek: K,
    config: KeyringConfig,
    clock: Arc<dyn Clock>,
    cache: Mutex<HashMap<Uuid, CacheEntry>>,
}

impl<K> fmt::Debug for Keyring<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keyring")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl<K: KeyEncryptionKey> Keyring<K> {
    /// Membuat keyring. TTL harus positif dan kapasitas minimal 1.
    pub fn new(kek: K, config: KeyringConfig, clock: Arc<dyn Clock>) -> Result<Self, CryptoError> {
        if config.cache_ttl <= Duration::ZERO || config.cache_capacity == 0 {
            return Err(CryptoError::InvalidConfig);
        }
        Ok(Self {
            kek,
            config,
            clock,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Membuat wadah baru: DEK dari KEK, baris `data_key` untuk di-`INSERT`, dan DEK siap pakai.
    /// Tidak ada panggilan database di sini; panggil sebelum membuka transaksi (KEPUTUSAN-125).
    pub async fn create_data_key(
        &self,
        container: Container,
    ) -> Result<(NewDataKey, DataKey), CryptoError> {
        let generated = self.kek.generate_dek().await?;
        let id = new_id();
        let dek = Arc::new(DataEncryptionKey::from_bytes(
            id,
            generated.plaintext.expose(),
        )?);
        self.cache_insert(id, container, Arc::clone(&dek));
        let row = NewDataKey {
            id,
            container_type: container.container_type,
            container_id: container.container_id,
            wrapped_dek: Sensitive::new(generated.wrapped.into_inner().into_bytes()),
            kek_ref: generated.kek_ref,
            retention_category: None,
            retention_anchor_at: None,
        };
        Ok((row, self.data_key(container, dek)))
    }

    /// Membuka DEK dari baris `data_key` untuk wadah `expected`.
    pub async fn open_data_key(
        &self,
        record: &DataKeyRecord,
        expected: Container,
    ) -> Result<DataKey, CryptoError> {
        let wrapped = match (&record.wrapped_dek, record.shredded_at) {
            (Some(w), None) => w,
            _ => {
                self.evict(record.id);
                return Err(CryptoError::KeyShredded);
            }
        };
        if record.container_type != expected.container_type.as_str()
            || record.container_id != expected.container_id
        {
            return Err(CryptoError::ContainerMismatch);
        }
        if let Some(dek) = self.cache_get(record.id, expected) {
            return Ok(self.data_key(expected, dek));
        }
        let wrapped =
            std::str::from_utf8(wrapped.expose()).map_err(|_| CryptoError::MalformedWrappedKey)?;
        let plaintext = self.kek.unwrap_dek(wrapped, &record.kek_ref).await?;
        let dek = Arc::new(DataEncryptionKey::from_bytes(
            record.id,
            plaintext.expose(),
        )?);
        self.cache_insert(record.id, expected, Arc::clone(&dek));
        Ok(self.data_key(expected, dek))
    }

    /// Membuang DEK dari cache (misalnya setelah pemusnahan diketahui).
    pub fn evict(&self, data_key_id: Uuid) {
        self.lock().remove(&data_key_id);
    }

    fn data_key(&self, container: Container, dek: Arc<DataEncryptionKey>) -> DataKey {
        DataKey {
            container,
            system: self.config.system,
            dek,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<Uuid, CacheEntry>> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn cache_get(&self, id: Uuid, expected: Container) -> Option<Arc<DataEncryptionKey>> {
        let now = self.clock.now();
        let mut cache = self.lock();
        let entry = cache.get(&id)?;
        if entry.expires_at <= now || entry.container != expected {
            cache.remove(&id);
            return None;
        }
        Some(Arc::clone(&entry.dek))
    }

    fn cache_insert(&self, id: Uuid, container: Container, dek: Arc<DataEncryptionKey>) {
        let now = self.clock.now();
        let Some(expires_at) = now.checked_add(self.config.cache_ttl) else {
            return; // di luar rentang waktu: tidak di-cache, DEK tetap dipakai sekali ini
        };
        let mut cache = self.lock();
        cache.retain(|_, e| e.expires_at > now);
        while cache.len() >= self.config.cache_capacity {
            let oldest = cache
                .iter()
                .min_by_key(|(_, e)| e.expires_at)
                .map(|(k, _)| *k);
            match oldest {
                Some(k) => {
                    cache.remove(&k);
                }
                None => break,
            }
        }
        cache.insert(
            id,
            CacheEntry {
                container,
                dek,
                expires_at,
            },
        );
    }

    #[cfg(test)]
    fn cached_len(&self) -> usize {
        self.lock().len()
    }
}

/// Future hasil [`DataKeyring`].
pub type DataKeyFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, CryptoError>> + Send + 'a>>;

/// [`Keyring`] sebagai objek trait (`Arc<dyn DataKeyring>`), untuk service yang tidak generik atas KEK.
pub trait DataKeyring: Send + Sync {
    /// Lihat [`Keyring::create_data_key`].
    fn create_data_key(&self, container: Container) -> DataKeyFuture<'_, (NewDataKey, DataKey)>;
    /// Lihat [`Keyring::open_data_key`].
    fn open_data_key<'a>(
        &'a self,
        record: &'a DataKeyRecord,
        expected: Container,
    ) -> DataKeyFuture<'a, DataKey>;
}

impl<K: KeyEncryptionKey + 'static> DataKeyring for Keyring<K> {
    fn create_data_key(&self, container: Container) -> DataKeyFuture<'_, (NewDataKey, DataKey)> {
        Box::pin(Keyring::create_data_key(self, container))
    }

    fn open_data_key<'a>(
        &'a self,
        record: &'a DataKeyRecord,
        expected: Container,
    ) -> DataKeyFuture<'a, DataKey> {
        Box::pin(Keyring::open_data_key(self, record, expected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::secret_bytes;
    use crate::primitives::fill_random;
    use crate::transit::GeneratedDek;
    use foundation::FixedClock;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use time::macros::datetime;

    /// KEK palsu di memori: menyimpan DEK per string terbungkus dan menghitung panggilan.
    #[derive(Default)]
    struct FakeKek {
        keys: Mutex<HashMap<String, Vec<u8>>>,
        generated: AtomicUsize,
        unwrapped: AtomicUsize,
    }

    impl KeyEncryptionKey for FakeKek {
        async fn generate_dek(&self) -> Result<GeneratedDek, CryptoError> {
            let n = self.generated.fetch_add(1, Ordering::SeqCst);
            let mut k = vec![0u8; 32];
            fill_random(&mut k).unwrap();
            let wrapped = format!("vault:v1:palsu-{n}");
            self.keys.lock().unwrap().insert(wrapped.clone(), k.clone());
            Ok(GeneratedDek {
                plaintext: secret_bytes(k),
                wrapped: Sensitive::new(wrapped),
                kek_ref: "openbao-transit:transit:uji:v1".into(),
            })
        }

        async fn unwrap_dek(
            &self,
            wrapped: &str,
            _kek_ref: &str,
        ) -> Result<SecretBytes, CryptoError> {
            self.unwrapped.fetch_add(1, Ordering::SeqCst);
            let keys = self.keys.lock().unwrap();
            keys.get(wrapped)
                .map(|k| secret_bytes(k.clone()))
                .ok_or(CryptoError::KmsRejected { status: 400 })
        }
    }

    const NIK: &[u8] = b"3200000000000001"; // sintetis
    const TTL_MIN: i64 = 10;

    fn keyring(capacity: usize) -> (Keyring<FakeKek>, Arc<FixedClock>) {
        let clock = Arc::new(FixedClock::new(datetime!(2027-03-01 08:00:00 +7)));
        let k = Keyring::new(
            FakeKek::default(),
            KeyringConfig {
                system: System::RuangSehat,
                cache_ttl: Duration::minutes(TTL_MIN),
                cache_capacity: capacity,
            },
            clock.clone(),
        )
        .unwrap();
        (k, clock)
    }

    fn record(row: &NewDataKey) -> DataKeyRecord {
        DataKeyRecord {
            id: row.id,
            container_type: row.container_type.as_str().to_string(),
            container_id: row.container_id,
            wrapped_dek: Some(Sensitive::new(row.wrapped_dek.expose().clone())),
            kek_ref: row.kek_ref.clone(),
            shredded_at: None,
        }
    }

    fn identity_doc() -> Container {
        Container::new(ContainerType::IdentityDocument, new_id())
    }

    #[tokio::test]
    async fn created_key_encrypts_and_reopened_key_decrypts() {
        let (kr, _) = keyring(8);
        let c = identity_doc();
        let (row, key) = kr.create_data_key(c).await.unwrap();
        assert_eq!(row.id, key.id());
        assert_eq!(row.container_type, ContainerType::IdentityDocument);
        assert_eq!(row.container_id, c.container_id);
        assert_eq!(row.wrapped_dek.expose(), b"vault:v1:palsu-0");
        let doc_id = c.container_id;
        let ct = key
            .seal(EncryptionContext::IDENTITY_DOCUMENT_NIK, doc_id, NIK)
            .unwrap();
        let reopened = kr.open_data_key(&record(&row), c).await.unwrap();
        let pt = reopened
            .open(EncryptionContext::IDENTITY_DOCUMENT_NIK, doc_id, &ct)
            .unwrap();
        assert_eq!(pt.expose().as_slice(), NIK);
        assert_eq!(
            kr.kek.unwrapped.load(Ordering::SeqCst),
            0,
            "DEK baru langsung dari cache"
        );
    }

    #[tokio::test]
    async fn cache_expires_after_ttl_and_key_is_unwrapped_again() {
        let (kr, clock) = keyring(8);
        let c = identity_doc();
        let (row, _) = kr.create_data_key(c).await.unwrap();
        let rec = record(&row);
        assert!(clock.advance(Duration::minutes(TTL_MIN - 1)));
        kr.open_data_key(&rec, c).await.unwrap();
        assert_eq!(kr.kek.unwrapped.load(Ordering::SeqCst), 0);
        assert!(clock.advance(Duration::minutes(1)));
        kr.open_data_key(&rec, c).await.unwrap();
        assert_eq!(
            kr.kek.unwrapped.load(Ordering::SeqCst),
            1,
            "kedaluwarsa tepat pada TTL"
        );
        kr.open_data_key(&rec, c).await.unwrap();
        assert_eq!(kr.kek.unwrapped.load(Ordering::SeqCst), 1, "di-cache ulang");
    }

    #[tokio::test]
    async fn shredded_key_is_rejected_even_when_cached_and_cache_is_dropped() {
        let (kr, _) = keyring(8);
        let c = identity_doc();
        let (row, key) = kr.create_data_key(c).await.unwrap();
        let ct = key
            .seal(
                EncryptionContext::IDENTITY_DOCUMENT_NIK,
                c.container_id,
                NIK,
            )
            .unwrap();
        drop(key);
        let mut shredded = record(&row);
        shredded.wrapped_dek = None;
        shredded.shredded_at = Some(datetime!(2027-03-02 00:00:00 UTC));
        assert_eq!(
            kr.open_data_key(&shredded, c).await.err(),
            Some(CryptoError::KeyShredded)
        );
        assert_eq!(kr.cached_len(), 0, "DEK dimusnahkan dibuang dari cache");
        // Bentuk tidak konsisten (hanya salah satu kolom) tetap dianggap musnah: gagal-tertutup.
        let mut half = record(&row);
        half.shredded_at = Some(datetime!(2027-03-02 00:00:00 UTC));
        assert_eq!(
            kr.open_data_key(&half, c).await.err(),
            Some(CryptoError::KeyShredded)
        );
        let mut half = record(&row);
        half.wrapped_dek = None;
        assert_eq!(
            kr.open_data_key(&half, c).await.err(),
            Some(CryptoError::KeyShredded)
        );
        assert!(!ct.is_empty());
    }

    #[tokio::test]
    async fn wrong_container_type_or_id_is_rejected_before_unwrapping() {
        let (kr, _) = keyring(8);
        let session = new_id();
        let chat = Container::new(ContainerType::ConsultationChat, session);
        let clinical = Container::new(ContainerType::ConsultationClinical, session);
        let (row, _) = kr.create_data_key(chat).await.unwrap();
        kr.evict(row.id);
        let rec = record(&row);
        // consultation_chat dan consultation_clinical berbagi container_id (DD §0.2).
        assert_eq!(
            kr.open_data_key(&rec, clinical).await.err(),
            Some(CryptoError::ContainerMismatch)
        );
        let other_session = Container::new(ContainerType::ConsultationChat, new_id());
        assert_eq!(
            kr.open_data_key(&rec, other_session).await.err(),
            Some(CryptoError::ContainerMismatch)
        );
        let mut unknown = record(&row);
        unknown.container_type = "rme_visit".into();
        assert_eq!(
            kr.open_data_key(&unknown, chat).await.err(),
            Some(CryptoError::ContainerMismatch)
        );
        assert_eq!(
            kr.kek.unwrapped.load(Ordering::SeqCst),
            0,
            "KEK tidak dipanggil"
        );
        assert!(kr.open_data_key(&rec, chat).await.is_ok());
    }

    #[tokio::test]
    async fn cached_key_is_not_served_for_another_container() {
        let (kr, _) = keyring(8);
        let session = new_id();
        let chat = Container::new(ContainerType::ConsultationChat, session);
        let (row, _) = kr.create_data_key(chat).await.unwrap();
        // Baris dipalsukan agar cocok dengan wadah lain: cache (berisi wadah chat) tidak dipakai.
        let mut forged = record(&row);
        forged.container_type = "consultation_clinical".into();
        let clinical = Container::new(ContainerType::ConsultationClinical, session);
        let key = kr.open_data_key(&forged, clinical).await.unwrap();
        assert_eq!(kr.kek.unwrapped.load(Ordering::SeqCst), 1);
        assert_eq!(key.container(), clinical);
    }

    #[tokio::test]
    async fn context_of_another_container_type_is_rejected() {
        let (kr, _) = keyring(8);
        let c = identity_doc();
        let (_, key) = kr.create_data_key(c).await.unwrap();
        let ctx = EncryptionContext::IDENTITY_VERIFICATION_REQUEST_STATUS_EVENT_NOTE; // wadah status_note
        assert_eq!(
            key.seal(ctx, new_id(), b"catatan").err(),
            Some(CryptoError::ContainerMismatch)
        );
        let ct = key
            .seal(
                EncryptionContext::IDENTITY_DOCUMENT_NIK,
                c.container_id,
                NIK,
            )
            .unwrap();
        assert_eq!(
            key.open(ctx, c.container_id, &ct).err(),
            Some(CryptoError::ContainerMismatch)
        );
    }

    #[tokio::test]
    async fn ciphertext_of_one_container_does_not_open_with_another_key() {
        let (kr, _) = keyring(8);
        let a = identity_doc();
        let b = identity_doc();
        let (_, ka) = kr.create_data_key(a).await.unwrap();
        let (_, kb) = kr.create_data_key(b).await.unwrap();
        let row = new_id();
        let ct = ka
            .seal(EncryptionContext::IDENTITY_DOCUMENT_NIK, row, NIK)
            .unwrap();
        assert_eq!(
            kb.open(EncryptionContext::IDENTITY_DOCUMENT_NIK, row, &ct)
                .err(),
            Some(CryptoError::DecryptionFailed)
        );
    }

    #[tokio::test]
    async fn cache_capacity_evicts_oldest_entry() {
        let (kr, clock) = keyring(2);
        let mut rows = Vec::new();
        for _ in 0..3 {
            let c = identity_doc();
            let (row, _) = kr.create_data_key(c).await.unwrap();
            rows.push((row, c));
            assert!(clock.advance(Duration::seconds(1)));
        }
        assert_eq!(kr.cached_len(), 2);
        kr.open_data_key(&record(&rows[0].0), rows[0].1)
            .await
            .unwrap();
        assert_eq!(
            kr.kek.unwrapped.load(Ordering::SeqCst),
            1,
            "entri tertua sudah dibuang"
        );
        kr.open_data_key(&record(&rows[2].0), rows[2].1)
            .await
            .unwrap();
        assert_eq!(kr.kek.unwrapped.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn non_utf8_wrapped_key_and_kek_failure_are_errors() {
        let (kr, _) = keyring(8);
        let c = identity_doc();
        let (row, _) = kr.create_data_key(c).await.unwrap();
        kr.evict(row.id);
        let mut bad = record(&row);
        bad.wrapped_dek = Some(Sensitive::new(vec![0xff, 0xfe]));
        assert_eq!(
            kr.open_data_key(&bad, c).await.err(),
            Some(CryptoError::MalformedWrappedKey)
        );
        let mut unknown = record(&row);
        unknown.wrapped_dek = Some(Sensitive::new(b"vault:v1:tidak-dikenal".to_vec()));
        assert_eq!(
            kr.open_data_key(&unknown, c).await.err(),
            Some(CryptoError::KmsRejected { status: 400 })
        );
    }

    #[test]
    fn invalid_config_is_rejected() {
        let clock: Arc<dyn Clock> = Arc::new(FixedClock::new(datetime!(2027-03-01 00:00:00 UTC)));
        for (ttl, cap) in [
            (Duration::ZERO, 1),
            (Duration::seconds(-1), 1),
            (Duration::minutes(1), 0),
        ] {
            let r = Keyring::new(
                FakeKek::default(),
                KeyringConfig {
                    system: System::RuangSehat,
                    cache_ttl: ttl,
                    cache_capacity: cap,
                },
                clock.clone(),
            );
            assert_eq!(r.err().map(|_| ()), Some(()));
        }
    }

    #[tokio::test]
    async fn debug_output_of_rows_and_keys_does_not_leak_secrets() {
        let (kr, _) = keyring(8);
        let (row, key) = kr.create_data_key(identity_doc()).await.unwrap();
        let out = format!("{row:?} {:?} {key:?} {kr:?}", record(&row));
        assert!(!out.contains("vault:v1"));
        assert!(!out.contains("palsu"));
        assert!(out.contains("[REDACTED]"));
    }
}
