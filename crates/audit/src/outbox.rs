//! Transaksi teraudit: pustaka memegang begin/commit dan menulis `audit_outbox` di transaksi yang sama
//! dengan aksi bisnis (ADR-009 butir 2, DD A19.1, KEPUTUSAN-15; F1-T23 setelah review keamanan BLOKIR-2).
//!
//! [`in_audited_tx`] memulai transaksi TINGKAT ATAS sendiri, menjalankan aksi bisnis pemanggil dengan
//! koneksi transaksi itu, memvalidasi dan memasking event yang dikembalikan aksi, menulisnya ke
//! `app.audit_outbox`, lalu commit. Aksi bisnis tidak pernah memegang objek transaksi tingkat atas,
//! sehingga tidak dapat meng-commit perubahan tanpa event: savepoint yang dibuka aksi (`conn.begin()`)
//! berada di dalam transaksi pustaka dan ikut batal bila penulisan event gagal. Tidak ada API publik
//! untuk menulis event ke transaksi milik pemanggil (pengganti `record(tx, ..)` dan ilustrasi `&mut tx`
//! di coding-standards §4.4).
//!
//! Satu-satunya jalan keluar yang tersisa adalah SQL kendali transaksi mentah (`COMMIT`, `ROLLBACK`, ...)
//! di dalam aksi. Dua lapis menutupnya: test arsitektur menolak literal SQL kendali transaksi di kode
//! non-test (aturan `tx-control`, crate arch-check), dan pustaka membandingkan ID transaksi
//! (`pg_current_xact_id()`) sebelum dan sesudah aksi; bila berbeda, event tidak ditulis, galat
//! `TransactionEscaped` dikembalikan, dan security log dicatat. Perubahan yang sudah di-commit lewat
//! SQL mentah itu tidak dapat dibatalkan pustaka; karena itu lapis pertama wajib.
//!
//! Role aplikasi hanya punya `INSERT` di `audit_outbox` (tanpa `SELECT`), sehingga query tidak memakai
//! `RETURNING`. Pesan galat database dibuang (dapat memuat isi baris di DETAIL, TD-03); yang dicatat
//! hanya entri security log berkode event dan tahap kegagalan (klarifikasi T-21).
//!
//! Pola (coding-standards §4.4): aksi yang mengubah data mengembalikan event-nya dari aksi; akses baca
//! data sensitif (E-05) memanggil `in_audited_tx` tanpa perubahan bisnis dan baru membaca/mendekripsi
//! setelah fungsi ini sukses (event sudah di-commit); aksi gagal/ditolak memanggilnya terpisah dengan
//! event berhasil-gagal, percobaan ulang oleh pemanggil (F1-T38).

use crate::error::{AuditError, AuditErrorKind};
use crate::event::{AuditEvent, AuditRecord};
use foundation::Clock;
use observability::security::{self, AuditWriteFailure};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Future aksi bisnis: hasil aksi dan event audit yang ditimbulkannya (minimal satu).
pub type AuditedWork<'c, T, E> =
    Pin<Box<dyn Future<Output = Result<(T, Vec<AuditEvent>), E>> + Send + 'c>>;

/// Galat transaksi teraudit. Setiap varian berarti transaksi sudah di-rollback (tidak ada yang di-commit),
/// kecuali `Audit` berjenis `TransactionEscaped` (lihat dokumentasi modul).
#[derive(Debug)]
pub enum AuditedTxError<E> {
    /// Aksi bisnis mengembalikan galat.
    Work(E),
    /// Event ditolak pustaka, penulisan `audit_outbox` gagal, atau transaksi diakhiri aksi.
    Audit(AuditError),
    /// `BEGIN`/`COMMIT` gagal. Hanya SQLSTATE yang dibawa (tanpa pesan/DETAIL yang dapat memuat isi baris).
    Database {
        /// Kode SQLSTATE bila ada (mis. `40001` untuk dicoba ulang).
        sqlstate: Option<String>,
    },
}

impl<E: fmt::Display> fmt::Display for AuditedTxError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work(e) => write!(f, "aksi gagal: {e}"),
            Self::Audit(e) => write!(f, "{e}"),
            Self::Database { sqlstate } => write!(
                f,
                "transaksi database gagal (SQLSTATE {})",
                sqlstate.as_deref().unwrap_or("-")
            ),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for AuditedTxError<E> {}

fn db_error<E>(e: &sqlx::Error) -> AuditedTxError<E> {
    AuditedTxError::Database {
        sqlstate: e
            .as_database_error()
            .and_then(|d| d.code().map(|c| c.into_owned())),
    }
}

/// Menjalankan `work` dalam transaksi tingkat atas milik pustaka dan menulis event yang dikembalikannya
/// ke `app.audit_outbox` sebelum commit. Sukses: aksi dan event ter-commit bersama, hasil aksi
/// dikembalikan. Gagal di tahap mana pun: semuanya di-rollback (ADR-009 butir 2, KEPUTUSAN-15).
///
/// `clock` = jam server aplikasi untuk `occurred_at` (ADR-009 butir 6). `work` menerima koneksi
/// transaksi; data yang dipakai di dalamnya dipindahkan (`move`) ke future.
pub async fn in_audited_tx<T, E, F>(
    pool: &PgPool,
    clock: &dyn Clock,
    work: F,
) -> Result<T, AuditedTxError<E>>
where
    T: Send,
    E: Send,
    F: for<'c> FnOnce(&'c mut PgConnection) -> AuditedWork<'c, T, E> + Send,
{
    let mut tx = pool.begin().await.map_err(|e| db_error(&e))?;
    let xid_before = match current_xid(&mut tx).await {
        Ok(x) => x,
        Err(e) => {
            abort(tx).await;
            return Err(db_error(&e));
        }
    };

    let (value, events) = match work(&mut tx).await {
        Ok(v) => v,
        Err(e) => {
            abort(tx).await;
            return Err(AuditedTxError::Work(e));
        }
    };

    let first_code = events.first().map(|e| e.code().as_str()).unwrap_or("none");
    if events.is_empty() {
        security::audit_write_failed(first_code, AuditWriteFailure::InvalidEvent);
        abort(tx).await;
        return Err(AuditedTxError::Audit(AuditError::new(
            AuditErrorKind::MissingField,
            "events",
        )));
    }
    let mut records = Vec::with_capacity(events.len());
    for event in events {
        let code = event.code();
        match event.finish(clock) {
            Ok(rec) => records.push(rec),
            Err(e) => {
                security::audit_write_failed(code.as_str(), AuditWriteFailure::InvalidEvent);
                abort(tx).await;
                return Err(AuditedTxError::Audit(e));
            }
        }
    }

    // Aksi tidak boleh mengakhiri transaksi pustaka (COMMIT/ROLLBACK mentah).
    match current_xid(&mut tx).await {
        Ok(x) if x == xid_before => {}
        _ => {
            security::audit_write_failed(first_code, AuditWriteFailure::TransactionEscaped);
            abort(tx).await;
            return Err(AuditedTxError::Audit(AuditError::new(
                AuditErrorKind::TransactionEscaped,
                "transaction",
            )));
        }
    }

    for rec in &records {
        if insert(&mut tx, rec).await.is_err() {
            security::audit_write_failed(rec.code().as_str(), AuditWriteFailure::StoreError);
            abort(tx).await;
            return Err(AuditedTxError::Audit(AuditError::new(
                AuditErrorKind::Write,
                "audit_outbox",
            )));
        }
    }
    tx.commit().await.map_err(|e| db_error(&e))?;
    Ok(value)
}

/// Menulis satu event yang **sudah dibentuk** ([`AuditEvent::finish`]) ke `app.audit_outbox` di
/// transaksi tersendiri, idempoten: event dengan ID yang sama yang sudah ada tidak ditulis ulang
/// (`ON CONFLICT DO NOTHING`). Untuk event berhasil-gagal yang dicoba ulang di luar aksi bisnis
/// (coding-standards §4.4 butir 3, F1-T38): event dan ID-nya dibentuk sekali, sehingga percobaan ulang
/// setelah commit yang balasannya hilang tidak menggandakan event. Bila baris sudah dipindah relay,
/// event yang ditulis ulang dilewati relay berdasarkan ID event (F1-T24). Tidak menerima transaksi milik
/// pemanggil.
pub async fn insert_finished(pool: &PgPool, record: &AuditRecord) -> Result<(), AuditError> {
    let result = sqlx::query!(
        "INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES ($1, $2, $3) \
         ON CONFLICT DO NOTHING",
        record.event_id(),
        record.occurred_at(),
        record.payload(),
    )
    .execute(pool)
    .await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            security::audit_write_failed(record.code().as_str(), AuditWriteFailure::StoreError);
            Err(AuditError::new(AuditErrorKind::Write, "audit_outbox"))
        }
    }
}

/// Rollback eksplisit. Bila rollback sendiri gagal (koneksi putus), koneksi dibuang pool dan server
/// membatalkan transaksi yang belum di-commit; tidak ada jalur commit yang tersisa.
async fn abort(tx: Transaction<'_, Postgres>) {
    let _rolled_back = tx.rollback().await;
}

async fn current_xid(conn: &mut PgConnection) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT pg_current_xact_id()::text AS "xid!""#)
        .fetch_one(conn)
        .await
}

async fn insert(conn: &mut PgConnection, rec: &AuditRecord) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES ($1, $2, $3)",
        rec.event_id(),
        rec.occurred_at(),
        rec.payload(),
    )
    .execute(conn)
    .await?;
    Ok(())
}
