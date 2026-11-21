//! Jalur tulis status entitas transaksi (ADR-008 butir 2-3, coding-standards §5.2; F1-T59, reference
//! implementation US-106/US-201).
//!
//! Satu transisi = satu pemanggilan [`transition`] di dalam transaksi use case (di Ruang Sehat:
//! `audit::in_audited_tx`, sehingga langkah (e) "tulis outbox audit" ikut transaksi yang sama):
//! 1. (a) baca status dan `version` terkini dari proyeksi ([`StatusStore::current`]);
//! 2. (b) validasi transisi terhadap aturan entitas ([`TransitionRules::allows`], tabel transisi PRD §3.6);
//! 3. (c) `INSERT` event riwayat dengan `version + 1` ([`StatusStore::append`]). Repository wajib memakai
//!    `ON CONFLICT ON CONSTRAINT <uq_(entitas_id, version)> DO NOTHING`: bila transaksi lain sudah (atau
//!    sedang) menulis versi yang sama, PostgreSQL menunggu transaksi itu selesai lalu tidak menyisipkan
//!    apa pun, dan transaksi pemanggil tetap dapat dipakai (tanpa galat 23505 yang membatalkan transaksi);
//! 4. bila kalah ([`Appended::Lost`]): status baru dibaca ulang dan transisi **dievaluasi ulang** dari
//!    status baru (AB-36). Tidak sah → [`TransitionError::NotAllowed`] (dipetakan pemanggil ke `4090001`
//!    dan dicatat dengan hasil gagal di transaksi tersendiri); masih sah → dicoba lagi, paling banyak
//!    [`MAX_ATTEMPTS`] kali, lalu [`TransitionError::Contended`];
//! 5. (d) `UPDATE` proyeksi dengan syarat `version` lama (atau `INSERT` untuk versi 1)
//!    ([`StatusStore::project`]). Proyeksi yang tidak berada di versi lama padahal event berhasil
//!    disisipkan berarti proyeksi menyimpang dari riwayat ([`TransitionError::ProjectionMismatch`]):
//!    pemanggil membatalkan transaksi dan mencatat kejadian keamanan (ADR-008 butir 4).
//!
//! Pustaka ini tidak memuat SQL (tabel milik modul pemilik, coding-standards §1.3) dan tidak memuat jenis
//! status entitas: keduanya disediakan modul lewat [`StatusStore`] dan [`TransitionRules`].

use std::fmt;
use std::future::Future;

/// Batas percobaan saat kalah bersaing menulis versi yang sama (langkah 4). Usulan teknis F1-T59: satu
/// percobaan ulang sudah cukup untuk dua penulis; tiga menahan antrean kecil tanpa putaran tak terbatas.
pub const MAX_ATTEMPTS: usize = 3;

/// Aturan transisi satu entitas untuk satu pelaku/konteks (misalnya "pasien membatalkan pengajuan
/// dokumen"). `from = None` = entitas belum punya status (event versi 1).
pub trait TransitionRules<S> {
    /// Apakah `from → to` sah untuk pelaku dan konteks aturan ini.
    fn allows(&self, from: Option<S>, to: S) -> bool;
}

/// Status terkini menurut proyeksi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Current<S> {
    /// Status.
    pub status: S,
    /// Versi (`>= 1`).
    pub version: i32,
}

/// Satu langkah transisi yang akan atau sudah ditulis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step<S> {
    /// Status sebelum (`None` untuk versi 1).
    pub from: Option<S>,
    /// Status sesudah.
    pub to: S,
    /// Versi event yang ditulis (`from` berada di `version - 1`).
    pub version: i32,
}

/// Hasil penyisipan event (langkah c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appended {
    /// Event tersisip dengan versi yang diminta.
    Inserted,
    /// Versi itu sudah dipakai transaksi lain (konflik `UNIQUE (<entitas>_id, version)`).
    Lost,
}

/// Penyimpanan riwayat dan proyeksi **satu** entitas di dalam transaksi pemanggil (diimplementasikan
/// repository modul pemilik tabel). Isi event selain status (alasan, catatan terenkripsi, pelaku,
/// `correlation_id`) dibawa implementasi.
pub trait StatusStore<S> {
    /// Galat penyimpanan (database).
    type Error;

    /// (a) Status dan versi terkini dari proyeksi; `None` bila entitas belum punya status.
    fn current(&mut self) -> impl Future<Output = Result<Option<Current<S>>, Self::Error>> + Send;

    /// (c) Menyisipkan event `step` dengan `ON CONFLICT ... DO NOTHING` atas `(entitas_id, version)`.
    fn append(
        &mut self,
        step: Step<S>,
    ) -> impl Future<Output = Result<Appended, Self::Error>> + Send;

    /// (d) Memajukan proyeksi ke `step`: `INSERT` bila `step.version == 1`, selain itu `UPDATE ... WHERE
    /// version = step.version - 1`. `false` = proyeksi tidak berada di versi lama.
    fn project(&mut self, step: Step<S>) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}

/// Galat transisi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError<S, E> {
    /// Transisi tidak sah dari status terkini (termasuk setelah kalah bersaing dan dievaluasi ulang).
    NotAllowed {
        /// Status terkini saat diputuskan.
        from: Option<S>,
        /// Status yang diminta.
        to: S,
    },
    /// Kalah bersaing [`MAX_ATTEMPTS`] kali berturut-turut.
    Contended,
    /// Event tersisip tetapi proyeksi tidak berada di versi lama: proyeksi menyimpang dari riwayat.
    ProjectionMismatch,
    /// Versi melewati batas `i32`.
    VersionOverflow,
    /// Galat penyimpanan.
    Store(E),
}

impl<S: fmt::Debug, E: fmt::Display> fmt::Display for TransitionError<S, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAllowed { from, to } => {
                write!(f, "transisi tidak sah: {from:?} -> {to:?}")
            }
            Self::Contended => f.write_str("transisi kalah bersaing berulang kali"),
            Self::ProjectionMismatch => f.write_str("proyeksi status menyimpang dari riwayat"),
            Self::VersionOverflow => f.write_str("versi status melewati batas"),
            Self::Store(e) => write!(f, "penyimpanan status gagal: {e}"),
        }
    }
}

impl<S: fmt::Debug, E: fmt::Debug + fmt::Display> std::error::Error for TransitionError<S, E> {}

/// Menjalankan satu transisi ke `to` atas entitas milik `store` menurut `rules` (langkah 1-5 di
/// dokumentasi crate). Sukses mengembalikan langkah yang tertulis. Galat apa pun berarti pemanggil wajib
/// membatalkan transaksinya (aksi teraudit mengembalikan galat, sehingga `in_audited_tx` me-rollback).
pub async fn transition<S, R, St>(
    store: &mut St,
    rules: &R,
    to: S,
) -> Result<Step<S>, TransitionError<S, St::Error>>
where
    S: Copy + Send + Sync,
    R: TransitionRules<S> + Sync,
    St: StatusStore<S> + Send,
{
    for _ in 0..MAX_ATTEMPTS {
        let current = store.current().await.map_err(TransitionError::Store)?;
        let from = current.map(|c| c.status);
        if !rules.allows(from, to) {
            return Err(TransitionError::NotAllowed { from, to });
        }
        let version = match current {
            None => 1,
            Some(c) => c
                .version
                .checked_add(1)
                .ok_or(TransitionError::VersionOverflow)?,
        };
        let step = Step { from, to, version };
        match store.append(step).await.map_err(TransitionError::Store)? {
            Appended::Lost => continue,
            Appended::Inserted => {}
        }
        if !store.project(step).await.map_err(TransitionError::Store)? {
            return Err(TransitionError::ProjectionMismatch);
        }
        return Ok(step);
    }
    Err(TransitionError::Contended)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum St {
        Open,
        Done,
        Cancelled,
    }

    /// Open (versi 1) -> Done | Cancelled; tidak ada transisi dari status akhir.
    struct Rules;

    impl TransitionRules<St> for Rules {
        fn allows(&self, from: Option<St>, to: St) -> bool {
            matches!(
                (from, to),
                (None, St::Open) | (Some(St::Open), St::Done | St::Cancelled)
            )
        }
    }

    /// Penyimpanan tiruan: riwayat + proyeksi, dengan "penulis lain" yang dapat menyela tepat sebelum
    /// penyisipan berikutnya (meniru transaksi lain yang commit lebih dulu).
    #[derive(Default)]
    struct Mem {
        events: Vec<Step<St>>,
        projection: Option<Current<St>>,
        interleave: VecDeque<St>,
        drift: bool,
        reads: usize,
    }

    impl Mem {
        fn commit(&mut self, to: St) {
            let from = self.projection.map(|c| c.status);
            let version = self.projection.map_or(1, |c| c.version + 1);
            self.events.push(Step { from, to, version });
            self.projection = Some(Current {
                status: to,
                version,
            });
        }
    }

    impl StatusStore<St> for Mem {
        type Error = &'static str;

        async fn current(&mut self) -> Result<Option<Current<St>>, Self::Error> {
            self.reads += 1;
            Ok(self.projection)
        }

        async fn append(&mut self, step: Step<St>) -> Result<Appended, Self::Error> {
            if let Some(other) = self.interleave.pop_front() {
                self.commit(other);
            }
            if self.events.iter().any(|e| e.version == step.version) {
                return Ok(Appended::Lost);
            }
            self.events.push(step);
            Ok(Appended::Inserted)
        }

        async fn project(&mut self, step: Step<St>) -> Result<bool, Self::Error> {
            if self.drift {
                return Ok(false);
            }
            let ok = match (self.projection, step.version) {
                (None, 1) => true,
                (Some(c), v) => c.version + 1 == v,
                _ => false,
            };
            if ok {
                self.projection = Some(Current {
                    status: step.to,
                    version: step.version,
                });
            }
            Ok(ok)
        }
    }

    #[tokio::test]
    async fn first_transition_writes_version_one_and_projection() {
        let mut m = Mem::default();
        let step = transition(&mut m, &Rules, St::Open).await.unwrap();
        assert_eq!(
            step,
            Step {
                from: None,
                to: St::Open,
                version: 1
            }
        );
        assert_eq!(
            m.projection,
            Some(Current {
                status: St::Open,
                version: 1
            })
        );
    }

    #[tokio::test]
    async fn invalid_transition_is_rejected_without_writing() {
        let mut m = Mem::default();
        m.commit(St::Open);
        m.commit(St::Done);
        let err = transition(&mut m, &Rules, St::Cancelled).await.unwrap_err();
        assert_eq!(
            err,
            TransitionError::NotAllowed {
                from: Some(St::Done),
                to: St::Cancelled
            }
        );
        assert_eq!(m.events.len(), 2);
    }

    // trace: US-201 AC5, US-106 AC5 (dua keputusan bersamaan: yang tersimpan lebih dulu berlaku)
    #[tokio::test]
    async fn losing_writer_is_reevaluated_from_new_status_and_rejected() {
        let mut m = Mem::default();
        m.commit(St::Open);
        // Transaksi lain menyimpan Done tepat sebelum kita menyisipkan Cancelled versi 2.
        m.interleave.push_back(St::Done);
        let err = transition(&mut m, &Rules, St::Cancelled).await.unwrap_err();
        assert_eq!(
            err,
            TransitionError::NotAllowed {
                from: Some(St::Done),
                to: St::Cancelled
            }
        );
        assert_eq!(m.reads, 2, "status dibaca ulang setelah kalah");
        assert_eq!(m.events.len(), 2, "event yang kalah tidak tersimpan");
        assert_eq!(m.projection.unwrap().status, St::Done);
    }

    #[tokio::test]
    async fn losing_writer_retries_when_still_allowed() {
        // Aturan longgar: Open -> Open juga sah, sehingga setelah kalah masih boleh dicoba lagi.
        struct Loose;
        impl TransitionRules<St> for Loose {
            fn allows(&self, _from: Option<St>, _to: St) -> bool {
                true
            }
        }
        let mut m = Mem::default();
        m.commit(St::Open);
        m.interleave.push_back(St::Open);
        let step = transition(&mut m, &Loose, St::Done).await.unwrap();
        assert_eq!(step.version, 3);
        assert_eq!(step.from, Some(St::Open));
    }

    #[tokio::test]
    async fn repeated_losses_end_as_contended() {
        struct Loose;
        impl TransitionRules<St> for Loose {
            fn allows(&self, _from: Option<St>, _to: St) -> bool {
                true
            }
        }
        let mut m = Mem::default();
        m.commit(St::Open);
        for _ in 0..MAX_ATTEMPTS {
            m.interleave.push_back(St::Open);
        }
        let err = transition(&mut m, &Loose, St::Done).await.unwrap_err();
        assert_eq!(err, TransitionError::Contended);
        assert_eq!(m.reads, MAX_ATTEMPTS);
    }

    #[tokio::test]
    async fn projection_drift_is_reported() {
        let mut m = Mem::default();
        m.commit(St::Open);
        m.drift = true;
        let err = transition(&mut m, &Rules, St::Done).await.unwrap_err();
        assert_eq!(err, TransitionError::ProjectionMismatch);
    }

    #[tokio::test]
    async fn version_overflow_is_an_error_not_a_wrap() {
        let mut m = Mem {
            projection: Some(Current {
                status: St::Open,
                version: i32::MAX,
            }),
            ..Mem::default()
        };
        let err = transition(&mut m, &Rules, St::Done).await.unwrap_err();
        assert_eq!(err, TransitionError::VersionOverflow);
    }
}
