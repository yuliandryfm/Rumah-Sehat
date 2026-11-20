//! Bentuk kanonis event audit dan hash berantai (ADR-009 butir 5, DD B1 `canonical_version`,
//! `prev_hash`, `event_hash`; KEPUTUSAN-102).
//!
//! `event_hash = SHA-256(prev_hash ‖ kanonis_v1(event))`, `prev_hash` event pertama = 32 byte nol.
//!
//! **Bentuk kanonis versi 1** (keputusan teknis F1-T24; dinaikkan lewat `canonical_version` bila berubah).
//! Deretan byte tanpa ambiguitas: label domain, lalu setiap kolom DD B1 dalam urutan tetap di bawah, setiap
//! nilai panjang-variabel diawali panjangnya (u32 big-endian), setiap kolom nullable diawali penanda
//! (`0x00` = NULL, `0x01` = ada). Kolom yang dicakup: semua kolom `audit_event` kecuali `prev_hash` dan
//! `event_hash` (prev_hash sudah menjadi awalan hash), ditambah pasangan (`subject_id`, `subject_kind`) dari
//! **semua** baris `audit_event_subject` ber-`event_id` dan `recorded_at` sama dengan event, apa pun
//! `subject_count`-nya (relay hanya menulis baris subjek bila `subject_count > 1`). Baris subjek yang tidak
//! tercakup bentuk kanonis (`recorded_at` berbeda, atau yatim tanpa event) diperiksa terpisah oleh
//! verifikasi (`verify`), bukan oleh hash.
//!
//! | Bagian | Pengodean |
//! | --- | --- |
//! | label | `b"ruang-sehat/audit_event/canonical/v1\n"` |
//! | `canonical_version` | i16 BE |
//! | `seq` | i64 BE |
//! | `event_id` | 16 byte |
//! | `recorded_at`, `occurred_at` | i64 BE mikrodetik sejak epoch Unix (UTC); presisi PostgreSQL `timestamptz` |
//! | teks wajib | u32 panjang + UTF-8 |
//! | teks/uuid/integer nullable | penanda + nilai |
//! | `text[]` nullable | penanda + u32 jumlah + tiap elemen (u32 panjang + UTF-8) |
//! | `source_ip` (`inet`) | `0x00` NULL, `0x04` + 4 oktet, `0x06` + 16 oktet |
//! | `before_values`, `after_values` (`jsonb`) | penanda + u32 panjang + JSON kanonis (kunci objek diurut byte, tanpa spasi, angka hanya bilangan bulat) |
//! | subjek | u32 jumlah + tiap subjek urut (`subject_id`, `subject_kind`): 16 byte + u32 panjang + UTF-8 |
//!
//! Nilai yang tersimpan harus dapat dibaca kembali persis: jam dibulatkan relay ke mikrodetik sebelum
//! ditulis, angka pecahan ditolak di JSON (pustaka audit hanya menghasilkan bilangan bulat, teks, boolean,
//! null), sehingga hasil hitung relay dan verifikator selalu sama untuk data yang tidak diubah.

use crate::error::{StoreError, StoreErrorKind};
use foundation::Uuid;
use serde_json::Value as Json;
use std::fmt;
use std::net::IpAddr;
use time::OffsetDateTime;

/// Versi bentuk kanonis yang dihasilkan modul ini (DD B1 `canonical_version`).
pub const CANONICAL_VERSION: i16 = 1;
/// Panjang hash SHA-256 (`ck_audit_event_*_hash`, 32 byte).
pub const HASH_LEN: usize = 32;
/// `prev_hash` event pertama rantai (KEPUTUSAN-102).
pub const GENESIS_PREV_HASH: [u8; HASH_LEN] = [0u8; HASH_LEN];

const DOMAIN_V1: &[u8] = b"ruang-sehat/audit_event/canonical/v1\n";

/// Hash event (32 byte).
pub type ChainHash = [u8; HASH_LEN];

/// Subjek data di `audit_event_subject` (DD B3).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventSubject {
    /// `subject_id`.
    pub subject_id: Uuid,
    /// `subject_kind` (`care_recipient`, `rme_patient`).
    pub subject_kind: String,
}

// Debug tanpa ID subjek (data pribadi, ADR-010 butir 1).
impl fmt::Debug for EventSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventSubject")
            .field("subject_kind", &self.subject_kind)
            .finish_non_exhaustive()
    }
}

/// Isi event (kolom DD B1 dari payload `audit_outbox`, sebelum diberi `seq` dan `recorded_at`).
#[derive(Clone, PartialEq)]
pub struct EventBody {
    /// `event_id` (UUIDv7 dari pustaka audit).
    pub event_id: Uuid,
    /// `occurred_at` (jam server aplikasi), presisi mikrodetik.
    pub occurred_at: OffsetDateTime,
    /// `event_code` (`e_01` … `e_18`).
    pub event_code: String,
    /// `action_name`.
    pub action_name: String,
    /// `actor_type`.
    pub actor_type: String,
    /// `actor_account_id`.
    pub actor_account_id: Option<Uuid>,
    /// `actor_account_type`.
    pub actor_account_type: Option<String>,
    /// `actor_roles` (NULL bila tidak ada peran).
    pub actor_roles: Option<Vec<String>>,
    /// `actor_partner_id`.
    pub actor_partner_id: Option<Uuid>,
    /// `object_type`.
    pub object_type: String,
    /// `object_id`.
    pub object_id: Option<String>,
    /// `object_version`.
    pub object_version: Option<i32>,
    /// `subject_id`: subjek utama (subjek pertama event), NULL bila tanpa subjek.
    pub subject_id: Option<Uuid>,
    /// `subject_count`.
    pub subject_count: i32,
    /// `tenant_id`.
    pub tenant_id: Option<Uuid>,
    /// `source_component`.
    pub source_component: String,
    /// `source_ip`.
    pub source_ip: Option<IpAddr>,
    /// `user_agent`.
    pub user_agent: Option<String>,
    /// `app_version`.
    pub app_version: Option<String>,
    /// `changed_fields`.
    pub changed_fields: Option<Vec<String>>,
    /// `before_values`.
    pub before_values: Option<Json>,
    /// `after_values`.
    pub after_values: Option<Json>,
    /// `disclosure_recipient`.
    pub disclosure_recipient: Option<String>,
    /// `disclosure_data_categories`.
    pub disclosure_data_categories: Option<Vec<String>>,
    /// `outcome`.
    pub outcome: String,
    /// `outcome_reason_code`.
    pub outcome_reason_code: Option<String>,
    /// `correlation_id`.
    pub correlation_id: Uuid,
    /// Baris `audit_event_subject`: semua subjek bila `subject_count > 1`, kosong bila tidak.
    pub subjects: Vec<EventSubject>,
}

// Debug tanpa kolom data pribadi (ID akun, IP, user agent, nilai before/after).
impl fmt::Debug for EventBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBody")
            .field("event_id", &self.event_id)
            .field("event_code", &self.event_code)
            .finish_non_exhaustive()
    }
}

/// Event di rantai: isi + posisi rantai.
#[derive(Clone, PartialEq)]
pub struct ChainEvent {
    /// `seq` (mulai 1, tanpa celah).
    pub seq: i64,
    /// `recorded_at` (jam server database audit).
    pub recorded_at: OffsetDateTime,
    /// `canonical_version`.
    pub canonical_version: i16,
    /// Isi event.
    pub body: EventBody,
}

impl fmt::Debug for ChainEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChainEvent")
            .field("seq", &self.seq)
            .field("event_id", &self.body.event_id)
            .finish_non_exhaustive()
    }
}

/// Membulatkan ke bawah ke mikrodetik (presisi `timestamptz`), sehingga nilai yang di-hash relay sama
/// dengan yang dibaca kembali verifikator.
pub fn truncate_to_micros(t: OffsetDateTime) -> OffsetDateTime {
    let nanos = t.nanosecond();
    t.replace_nanosecond(nanos - nanos % 1_000).unwrap_or(t)
}

/// `event_hash = SHA-256(prev_hash ‖ kanonis(event))`.
pub fn event_hash(prev_hash: &ChainHash, event: &ChainEvent) -> Result<ChainHash, StoreError> {
    let canonical = canonical_bytes(event)?;
    let mut input = Vec::with_capacity(HASH_LEN + canonical.len());
    input.extend_from_slice(prev_hash);
    input.extend_from_slice(&canonical);
    Ok(crypto::sha256(&input))
}

/// Bentuk kanonis versi 1 (lihat dokumentasi modul). Versi lain ditolak.
pub fn canonical_bytes(event: &ChainEvent) -> Result<Vec<u8>, StoreError> {
    if event.canonical_version != CANONICAL_VERSION {
        return Err(StoreError::new(
            StoreErrorKind::Canonical,
            "canonical_version",
        ));
    }
    let b = &event.body;
    let mut w = Writer::default();
    w.raw(DOMAIN_V1);
    w.raw(&event.canonical_version.to_be_bytes());
    w.raw(&event.seq.to_be_bytes());
    w.raw(b.event_id.as_bytes());
    w.ts(event.recorded_at, "recorded_at")?;
    w.ts(b.occurred_at, "occurred_at")?;
    w.text(&b.event_code)?;
    w.text(&b.action_name)?;
    w.text(&b.actor_type)?;
    w.opt_uuid(b.actor_account_id);
    w.opt_text(b.actor_account_type.as_deref())?;
    w.opt_text_array(b.actor_roles.as_deref())?;
    w.opt_uuid(b.actor_partner_id);
    w.text(&b.object_type)?;
    w.opt_text(b.object_id.as_deref())?;
    w.opt_i32(b.object_version);
    w.opt_uuid(b.subject_id);
    w.raw(&b.subject_count.to_be_bytes());
    w.opt_uuid(b.tenant_id);
    w.text(&b.source_component)?;
    w.opt_ip(b.source_ip);
    w.opt_text(b.user_agent.as_deref())?;
    w.opt_text(b.app_version.as_deref())?;
    w.opt_text_array(b.changed_fields.as_deref())?;
    w.opt_json(b.before_values.as_ref())?;
    w.opt_json(b.after_values.as_ref())?;
    w.opt_text(b.disclosure_recipient.as_deref())?;
    w.opt_text_array(b.disclosure_data_categories.as_deref())?;
    w.text(&b.outcome)?;
    w.opt_text(b.outcome_reason_code.as_deref())?;
    w.raw(b.correlation_id.as_bytes());
    let mut subjects: Vec<&EventSubject> = b.subjects.iter().collect();
    subjects.sort();
    w.len(subjects.len())?;
    for s in subjects {
        w.raw(s.subject_id.as_bytes());
        w.text(&s.subject_kind)?;
    }
    Ok(w.buf)
}

#[derive(Default)]
struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn raw(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    fn len(&mut self, n: usize) -> Result<(), StoreError> {
        let n =
            u32::try_from(n).map_err(|_| StoreError::new(StoreErrorKind::Canonical, "length"))?;
        self.raw(&n.to_be_bytes());
        Ok(())
    }

    fn text(&mut self, s: &str) -> Result<(), StoreError> {
        self.len(s.len())?;
        self.raw(s.as_bytes());
        Ok(())
    }

    fn present(&mut self, yes: bool) {
        self.buf.push(u8::from(yes));
    }

    fn opt_text(&mut self, s: Option<&str>) -> Result<(), StoreError> {
        self.present(s.is_some());
        match s {
            Some(s) => self.text(s),
            None => Ok(()),
        }
    }

    fn opt_uuid(&mut self, u: Option<Uuid>) {
        self.present(u.is_some());
        if let Some(u) = u {
            self.raw(u.as_bytes());
        }
    }

    fn opt_i32(&mut self, v: Option<i32>) {
        self.present(v.is_some());
        if let Some(v) = v {
            self.raw(&v.to_be_bytes());
        }
    }

    fn opt_text_array(&mut self, items: Option<&[String]>) -> Result<(), StoreError> {
        self.present(items.is_some());
        if let Some(items) = items {
            self.len(items.len())?;
            for s in items {
                self.text(s)?;
            }
        }
        Ok(())
    }

    fn opt_ip(&mut self, ip: Option<IpAddr>) {
        match ip {
            None => self.buf.push(0x00),
            Some(IpAddr::V4(v4)) => {
                self.buf.push(0x04);
                self.raw(&v4.octets());
            }
            Some(IpAddr::V6(v6)) => {
                self.buf.push(0x06);
                self.raw(&v6.octets());
            }
        }
    }

    fn opt_json(&mut self, v: Option<&Json>) -> Result<(), StoreError> {
        self.present(v.is_some());
        if let Some(v) = v {
            let mut text = String::new();
            canonical_json(v, &mut text)?;
            self.text(&text)?;
        }
        Ok(())
    }

    fn ts(&mut self, t: OffsetDateTime, what: &'static str) -> Result<(), StoreError> {
        let nanos = t.unix_timestamp_nanos();
        if nanos % 1_000 != 0 {
            // Presisi di bawah mikrodetik tidak dapat disimpan timestamptz: hash tidak akan cocok.
            return Err(StoreError::new(StoreErrorKind::Canonical, what));
        }
        let micros = i64::try_from(nanos / 1_000)
            .map_err(|_| StoreError::new(StoreErrorKind::Canonical, what))?;
        self.raw(&micros.to_be_bytes());
        Ok(())
    }
}

/// JSON kanonis: kunci objek diurut byte, tanpa spasi, angka hanya bilangan bulat (i64/u64), teks
/// di-escape serde_json. Angka pecahan ditolak karena tidak dijamin kembali persis dari `jsonb`.
pub fn canonical_json(v: &Json, out: &mut String) -> Result<(), StoreError> {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.push_str(&i.to_string());
            } else if let Some(u) = n.as_u64() {
                out.push_str(&u.to_string());
            } else {
                return Err(StoreError::new(StoreErrorKind::Canonical, "json_number"));
            }
        }
        Json::String(s) => {
            let quoted = serde_json::to_string(s)
                .map_err(|_| StoreError::new(StoreErrorKind::Canonical, "json_string"))?;
            out.push_str(&quoted);
        }
        Json::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical_json(item, out)?;
            }
            out.push(']');
        }
        Json::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let quoted = serde_json::to_string(k)
                    .map_err(|_| StoreError::new(StoreErrorKind::Canonical, "json_key"))?;
                out.push_str(&quoted);
                out.push(':');
                if let Some(item) = map.get(k) {
                    canonical_json(item, out)?;
                }
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Heksadesimal huruf kecil (untuk berkas jangkar dan log; hash bukan data pribadi).
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from(
            DIGITS.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
        ));
        s.push(char::from(
            DIGITS.get(usize::from(b & 0x0f)).copied().unwrap_or(b'0'),
        ));
    }
    s
}

/// Kebalikan [`hex`] untuk hash 32 byte; selain itu `None`.
pub fn parse_hash_hex(s: &str) -> Option<ChainHash> {
    if s.len() != HASH_LEN * 2 || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; HASH_LEN];
    for (slot, pair) in out.iter_mut().zip(s.as_bytes().chunks(2)) {
        let pair = std::str::from_utf8(pair).ok()?;
        *slot = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(out)
}

/// Mengubah `bytea` dari database menjadi hash 32 byte.
pub fn to_hash(bytes: &[u8]) -> Option<ChainHash> {
    <ChainHash>::try_from(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use time::macros::datetime;

    fn uuid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn body() -> EventBody {
        EventBody {
            event_id: uuid(1),
            occurred_at: datetime!(2027-03-01 10:00:00.123456 UTC),
            event_code: "e_07".into(),
            action_name: "identity_verification_approved".into(),
            actor_type: "account".into(),
            actor_account_id: Some(uuid(2)),
            actor_account_type: Some("rs_admin".into()),
            actor_roles: Some(vec!["verifier".into()]),
            actor_partner_id: None,
            object_type: "care_recipient".into(),
            object_id: Some(uuid(3).hyphenated().to_string()),
            object_version: None,
            subject_id: Some(uuid(3)),
            subject_count: 1,
            tenant_id: None,
            source_component: "backoffice".into(),
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 23))),
            user_agent: Some("Mozilla/5.0 uji".into()),
            app_version: None,
            changed_fields: Some(vec!["status".into(), "nik".into()]),
            before_values: Some(json!({"status": "pending"})),
            after_values: Some(json!({"status": "verified", "nik": {"last4": "8901"}})),
            disclosure_recipient: None,
            disclosure_data_categories: None,
            outcome: "success".into(),
            outcome_reason_code: None,
            correlation_id: uuid(4),
            subjects: Vec::new(),
        }
    }

    fn event() -> ChainEvent {
        ChainEvent {
            seq: 1,
            recorded_at: datetime!(2027-03-01 10:00:01 UTC),
            canonical_version: CANONICAL_VERSION,
            body: body(),
        }
    }

    #[test]
    fn hash_is_deterministic_and_chains_on_prev_hash() {
        let e = event();
        let h1 = event_hash(&GENESIS_PREV_HASH, &e).unwrap();
        assert_eq!(h1, event_hash(&GENESIS_PREV_HASH, &e).unwrap());
        let other_prev = [1u8; HASH_LEN];
        assert_ne!(h1, event_hash(&other_prev, &e).unwrap());
        // Hash = SHA-256(prev ‖ kanonis).
        let mut input = GENESIS_PREV_HASH.to_vec();
        input.extend(canonical_bytes(&e).unwrap());
        assert_eq!(h1, crypto::sha256(&input));
    }

    #[test]
    fn every_column_changes_the_hash() {
        let base = event_hash(&GENESIS_PREV_HASH, &event()).unwrap();
        type Mutation = Box<dyn Fn(&mut ChainEvent)>;
        let mutations: Vec<Mutation> = vec![
            Box::new(|e| e.seq = 2),
            Box::new(|e| e.recorded_at = datetime!(2027-03-01 10:00:02 UTC)),
            Box::new(|e| e.body.event_id = uuid(9)),
            Box::new(|e| e.body.occurred_at = datetime!(2027-03-01 10:00:00.123457 UTC)),
            Box::new(|e| e.body.event_code = "e_08".into()),
            Box::new(|e| e.body.action_name = "x".into()),
            Box::new(|e| e.body.actor_type = "system".into()),
            Box::new(|e| e.body.actor_account_id = None),
            Box::new(|e| e.body.actor_account_type = None),
            Box::new(|e| e.body.actor_roles = Some(vec![])),
            Box::new(|e| e.body.actor_roles = None),
            Box::new(|e| e.body.actor_partner_id = Some(uuid(5))),
            Box::new(|e| e.body.object_type = "patient".into()),
            Box::new(|e| e.body.object_id = None),
            Box::new(|e| e.body.object_version = Some(1)),
            Box::new(|e| e.body.subject_id = None),
            Box::new(|e| e.body.subject_count = 2),
            Box::new(|e| e.body.tenant_id = Some(uuid(6))),
            Box::new(|e| e.body.source_component = "worker".into()),
            Box::new(|e| e.body.source_ip = Some(IpAddr::V6(Ipv6Addr::LOCALHOST))),
            Box::new(|e| e.body.source_ip = None),
            Box::new(|e| e.body.user_agent = None),
            Box::new(|e| e.body.app_version = Some("1.0.0".into())),
            Box::new(|e| e.body.changed_fields = Some(vec!["status".into()])),
            Box::new(|e| e.body.before_values = Some(json!({"status": "rejected"}))),
            Box::new(|e| e.body.after_values = None),
            Box::new(|e| e.body.disclosure_recipient = Some("satusehat".into())),
            Box::new(|e| e.body.disclosure_data_categories = Some(vec!["identity".into()])),
            Box::new(|e| e.body.outcome = "failure".into()),
            Box::new(|e| e.body.outcome_reason_code = Some("access_denied".into())),
            Box::new(|e| e.body.correlation_id = uuid(7)),
            Box::new(|e| {
                e.body.subjects = vec![EventSubject {
                    subject_id: uuid(3),
                    subject_kind: "care_recipient".into(),
                }]
            }),
        ];
        for (i, m) in mutations.iter().enumerate() {
            let mut e = event();
            m(&mut e);
            assert_ne!(
                event_hash(&GENESIS_PREV_HASH, &e).unwrap(),
                base,
                "mutasi ke-{i} tidak mengubah hash"
            );
        }
    }

    #[test]
    fn length_prefixes_prevent_field_boundary_shifts() {
        // "ab" + "c" vs "a" + "bc" harus berbeda.
        let mut a = event();
        a.body.action_name = "ab".into();
        a.body.actor_type = "c".into();
        let mut b = event();
        b.body.action_name = "a".into();
        b.body.actor_type = "bc".into();
        assert_ne!(canonical_bytes(&a).unwrap(), canonical_bytes(&b).unwrap());
    }

    #[test]
    fn subjects_and_json_keys_are_order_independent() {
        let s1 = EventSubject {
            subject_id: uuid(10),
            subject_kind: "care_recipient".into(),
        };
        let s2 = EventSubject {
            subject_id: uuid(11),
            subject_kind: "care_recipient".into(),
        };
        let mut a = event();
        a.body.subjects = vec![s1.clone(), s2.clone()];
        let mut b = event();
        b.body.subjects = vec![s2, s1];
        assert_eq!(canonical_bytes(&a).unwrap(), canonical_bytes(&b).unwrap());

        let mut out = String::new();
        canonical_json(&json!({"b": 1, "a": [true, null, "x\"y"]}), &mut out).unwrap();
        assert_eq!(out, r#"{"a":[true,null,"x\"y"],"b":1}"#);
    }

    #[test]
    fn fractional_numbers_and_sub_micro_timestamps_are_rejected() {
        let mut e = event();
        e.body.after_values = Some(json!({"x": 1.5}));
        assert_eq!(
            canonical_bytes(&e).unwrap_err().kind(),
            StoreErrorKind::Canonical
        );
        let mut e = event();
        e.body.occurred_at = datetime!(2027-03-01 10:00:00.123456789 UTC);
        assert!(canonical_bytes(&e).is_err());
        assert_eq!(
            truncate_to_micros(datetime!(2027-03-01 10:00:00.123456789 UTC)),
            datetime!(2027-03-01 10:00:00.123456 UTC)
        );
        let mut e = event();
        e.canonical_version = 2;
        assert!(canonical_bytes(&e).is_err());
    }

    #[test]
    fn hex_roundtrip() {
        let h = event_hash(&GENESIS_PREV_HASH, &event()).unwrap();
        assert_eq!(parse_hash_hex(&hex(&h)), Some(h));
        assert_eq!(parse_hash_hex("zz"), None);
        assert_eq!(hex(&[0x0f, 0xa0]), "0fa0");
    }
}
