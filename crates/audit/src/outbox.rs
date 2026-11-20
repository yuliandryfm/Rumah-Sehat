//! Penulisan event ke `audit_outbox` di transaksi milik pemanggil (ADR-009 butir 2, DD A19.1, F1-T23).
//!
//! [`record`] mengambil alih transaksi pemanggil dan mengembalikannya hanya bila event tertulis. Bila
//! event ditolak pustaka atau `INSERT` gagal, transaksi di-rollback di sini dan tidak dikembalikan,
//! sehingga perubahan bisnis di transaksi yang sama tidak mungkin di-commit (KEPUTUSAN-15): aksi batal
//! dan pemanggil meneruskan galat ke pengguna.
//!
//! Role aplikasi hanya punya `INSERT` di `audit_outbox` (tanpa `SELECT`), sehingga query tidak memakai
//! `RETURNING`. Pesan galat database dibuang (dapat memuat isi baris di DETAIL, TD-03); yang dicatat
//! hanya entri security log berkode event dan tahap kegagalan (klarifikasi T-21).
//!
//! Pola pemakaian lain (ADR-009 butir 2, coding-standards §4.4): akses baca data sensitif (E-05) memakai
//! transaksi sendiri yang di-commit sebelum data dikirim; aksi gagal/ditolak memakai transaksi tersendiri
//! dengan percobaan ulang oleh pemanggil (F1-T38).

use crate::error::{AuditError, AuditErrorKind};
use crate::event::{AuditEvent, AuditRecord};
use foundation::Clock;
use observability::security::{self, AuditWriteFailure};
use sqlx::{PgConnection, Postgres, Transaction};

/// Memvalidasi dan memasking `event`, lalu menulisnya ke `app.audit_outbox` di transaksi `tx`.
///
/// Sukses: transaksi dikembalikan untuk dilanjutkan dan di-commit pemanggil. Gagal: transaksi sudah
/// di-rollback dan galat dikembalikan; perubahan bisnis di transaksi itu ikut batal.
pub async fn record<'c>(
    mut tx: Transaction<'c, Postgres>,
    event: AuditEvent,
    clock: &dyn Clock,
) -> Result<Transaction<'c, Postgres>, AuditError> {
    let code = event.code();
    let rec = match event.finish(clock) {
        Ok(rec) => rec,
        Err(e) => {
            security::audit_write_failed(code.as_str(), AuditWriteFailure::InvalidEvent);
            abort(tx).await;
            return Err(e);
        }
    };
    if insert(&mut tx, &rec).await.is_err() {
        security::audit_write_failed(code.as_str(), AuditWriteFailure::StoreError);
        abort(tx).await;
        return Err(AuditError::new(AuditErrorKind::Write, "audit_outbox"));
    }
    Ok(tx)
}

/// Rollback eksplisit. Bila rollback sendiri gagal (koneksi putus), koneksi dibuang pool dan server
/// membatalkan transaksi yang belum di-commit; tidak ada jalur commit yang tersisa.
async fn abort(tx: Transaction<'_, Postgres>) {
    let _rolled_back = tx.rollback().await;
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
