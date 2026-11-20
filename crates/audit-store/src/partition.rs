//! Pemantauan partisi masa depan `audit_event` (skema fisik §6.2 butir 2, KEPUTUSAN-100, TD-12; F1-T25).
//!
//! Partisi dibuat migrasi jauh ke depan (2026-11 s.d. 2029-12) dan tidak ada partisi `DEFAULT`: tanpa
//! partisi bulan berjalan, `INSERT` relay gagal dan event tertahan di outbox. `audit-verify` menghitung
//! jumlah partisi bulanan berurutan **sesudah** partisi bulan berjalan dan menulisnya sebagai metrik di
//! application log (`audit_partition_runway`, field `future_partitions`); bila kurang dari
//! [`MIN_FUTURE_PARTITIONS`] atau bulan berjalan tidak tercakup, security log `audit_partition_runway_low`.
//! Ini pemantauan production, bukan test berbasis tanggal hari ini (§6.2 butir 2, review N09).

use crate::error::{StoreError, StoreErrorKind};
use foundation::Clock;
use sqlx::PgConnection;
use time::OffsetDateTime;

/// Ambang peringatan (§6.2 butir 2): kurang dari 6 partisi bulanan masa depan.
pub const MIN_FUTURE_PARTITIONS: u32 = 6;

/// Rentang satu partisi `[from, to)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionRange {
    /// Batas bawah (inklusif).
    pub from: OffsetDateTime,
    /// Batas atas (eksklusif).
    pub to: OffsetDateTime,
}

/// Cakupan partisi dari waktu sekarang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionRunway {
    /// Ada partisi yang mencakup waktu sekarang.
    pub current_covered: bool,
    /// Jumlah partisi berurutan sesudah partisi bulan berjalan.
    pub future_partitions: u32,
    /// Akhir cakupan berurutan (eksklusif), bila bulan berjalan tercakup.
    pub covered_until: Option<OffsetDateTime>,
}

impl PartitionRunway {
    /// Perlu peringatan (§6.2 butir 2).
    pub fn is_low(&self) -> bool {
        !self.current_covered || self.future_partitions < MIN_FUTURE_PARTITIONS
    }
}

/// Menghitung cakupan dari daftar rentang (murni).
pub fn runway(ranges: &[PartitionRange], now: OffsetDateTime) -> PartitionRunway {
    let mut sorted: Vec<PartitionRange> = ranges.to_vec();
    sorted.sort_by_key(|r| r.from);
    let Some(current) = sorted.iter().position(|r| r.from <= now && now < r.to) else {
        return PartitionRunway {
            current_covered: false,
            future_partitions: 0,
            covered_until: None,
        };
    };
    let mut until = sorted.get(current).map(|r| r.to);
    let mut future = 0u32;
    for r in sorted.iter().skip(current + 1) {
        if Some(r.from) != until {
            break;
        }
        future = future.saturating_add(1);
        until = Some(r.to);
    }
    PartitionRunway {
        current_covered: true,
        future_partitions: future,
        covered_until: until,
    }
}

/// Rentang semua partisi `audit.audit_event` dari katalog (batas bentuk
/// `FOR VALUES FROM ('…') TO ('…')`, dibaca lewat `pg_get_expr`); partisi `DEFAULT` dilewati.
pub async fn partition_ranges(conn: &mut PgConnection) -> Result<Vec<PartitionRange>, StoreError> {
    let rows = sqlx::query!(
        r#"SELECT split_part(pg_get_expr(c.relpartbound, c.oid), '''', 2)::timestamptz AS "range_from!",
                  split_part(pg_get_expr(c.relpartbound, c.oid), '''', 4)::timestamptz AS "range_to!"
             FROM pg_catalog.pg_inherits i
             JOIN pg_catalog.pg_class c ON c.oid = i.inhrelid
            WHERE i.inhparent = 'audit.audit_event'::regclass
              AND pg_get_expr(c.relpartbound, c.oid) <> 'DEFAULT'"#
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| StoreError::db("partition_ranges", &e))?;
    if rows.is_empty() {
        return Err(StoreError::new(StoreErrorKind::Database, "no_partitions"));
    }
    Ok(rows
        .into_iter()
        .map(|r| PartitionRange {
            from: r.range_from,
            to: r.range_to,
        })
        .collect())
}

/// Membaca katalog, menulis metrik `audit_partition_runway`, dan security log bila di bawah ambang.
pub async fn check_partition_runway(
    conn: &mut PgConnection,
    clock: &dyn Clock,
) -> Result<PartitionRunway, StoreError> {
    let ranges = partition_ranges(conn).await?;
    let r = runway(&ranges, clock.now());
    tracing::info!(
        future_partitions = r.future_partitions,
        current_covered = r.current_covered,
        min_future_partitions = MIN_FUTURE_PARTITIONS,
        "audit_partition_runway"
    );
    if r.is_low() {
        observability::security::audit_partition_runway_low(r.future_partitions, r.current_covered);
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use time::{Date, Month, Time, UtcOffset};

    /// Partisi bulanan seperti migrasi (2026-11 s.d. 2029-12, KEPUTUSAN-100).
    fn monthly(from_year: i32, from_month: u8, count: u32) -> Vec<PartitionRange> {
        let mut out = Vec::new();
        let (mut y, mut m) = (from_year, from_month);
        let start = |y: i32, m: u8| {
            Date::from_calendar_date(y, Month::try_from(m).unwrap(), 1)
                .unwrap()
                .with_time(Time::MIDNIGHT)
                .assume_offset(UtcOffset::UTC)
        };
        for _ in 0..count {
            let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
            out.push(PartitionRange {
                from: start(y, m),
                to: start(ny, nm),
            });
            (y, m) = (ny, nm);
        }
        out
    }

    #[test]
    fn runway_counts_contiguous_future_months() {
        let p = monthly(2026, 11, 38);
        let r = runway(&p, datetime!(2027-03-15 08:00 UTC));
        assert!(r.current_covered);
        assert_eq!(r.future_partitions, 33); // 2027-04 … 2029-12
        assert_eq!(r.covered_until, Some(datetime!(2030-01-01 00:00 UTC)));
        assert!(!r.is_low());
    }

    #[test]
    fn runway_below_six_months_is_low_and_end_of_range_is_uncovered() {
        let p = monthly(2026, 11, 38);
        let r = runway(&p, datetime!(2029-07-31 23:59:59 UTC));
        assert_eq!(r.future_partitions, 5);
        assert!(r.is_low());
        let r = runway(&p, datetime!(2029-06-01 00:00 UTC));
        assert_eq!(r.future_partitions, 6);
        assert!(!r.is_low());
        let r = runway(&p, datetime!(2030-01-01 00:00 UTC));
        assert_eq!(
            r,
            PartitionRunway {
                current_covered: false,
                future_partitions: 0,
                covered_until: None
            }
        );
        assert!(r.is_low());
    }

    #[test]
    fn runway_stops_at_a_missing_month() {
        let mut p = monthly(2027, 1, 12);
        p.remove(4); // 2027-05 hilang
        let r = runway(&p, datetime!(2027-02-10 00:00 UTC));
        assert_eq!(r.future_partitions, 2); // 2027-03, 2027-04
        assert!(r.is_low());
    }
}
