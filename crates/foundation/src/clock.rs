//! Jam yang dapat diinjeksi. Waktu selalu UTC (ADR-017); konversi zona hanya di tepi presentasi.

use std::sync::Mutex;
use time::{Duration, OffsetDateTime, UtcOffset};

/// Sumber waktu. Service dan job menerima `&dyn Clock` atau generik, tidak memanggil jam sistem langsung.
pub trait Clock: Send + Sync {
    /// Waktu sekarang dalam UTC.
    fn now(&self) -> OffsetDateTime;
}

/// Jam sistem. Satu-satunya tempat yang boleh memanggil jam sistem (KEPUTUSAN-131).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods)] // satu-satunya pemanggilan jam sistem yang sah (KEPUTUSAN-131)
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// Jam tetap untuk test; dapat digeser manual.
#[derive(Debug)]
pub struct FixedClock {
    at: Mutex<OffsetDateTime>,
}

impl FixedClock {
    /// Membuat jam yang selalu menunjuk `at` (dinormalkan ke UTC).
    pub fn new(at: OffsetDateTime) -> Self {
        Self {
            at: Mutex::new(at.to_offset(UtcOffset::UTC)),
        }
    }

    /// Mengatur waktu ke `at`.
    pub fn set(&self, at: OffsetDateTime) {
        *self.at.lock().unwrap_or_else(|e| e.into_inner()) = at.to_offset(UtcOffset::UTC);
    }

    /// Menggeser waktu sebesar `by`; mengembalikan `false` bila hasilnya di luar rentang yang didukung.
    pub fn advance(&self, by: Duration) -> bool {
        let mut guard = self.at.lock().unwrap_or_else(|e| e.into_inner());
        match guard.checked_add(by) {
            Some(next) => {
                *guard = next;
                true
            }
            None => false,
        }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        *self.at.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn fixed_clock_returns_same_instant() {
        let c = FixedClock::new(datetime!(2027-03-01 00:00:00 +7));
        assert_eq!(c.now(), datetime!(2027-02-28 17:00:00 UTC));
        assert_eq!(c.now(), c.now());
        assert_eq!(c.now().offset(), UtcOffset::UTC);
    }

    #[test]
    fn fixed_clock_set_and_advance() {
        let c = FixedClock::new(datetime!(2027-03-01 00:00:00 UTC));
        assert!(c.advance(Duration::hours(1)));
        assert_eq!(c.now(), datetime!(2027-03-01 01:00:00 UTC));
        c.set(datetime!(2030-01-01 00:00:00 UTC));
        assert_eq!(c.now(), datetime!(2030-01-01 00:00:00 UTC));
    }

    #[test]
    fn fixed_clock_advance_out_of_range_is_rejected() {
        let c = FixedClock::new(datetime!(2027-03-01 00:00:00 UTC));
        assert!(!c.advance(Duration::MAX));
        assert_eq!(c.now(), datetime!(2027-03-01 00:00:00 UTC));
    }

    #[test]
    fn system_clock_is_utc_and_usable_as_trait_object() {
        let c: &dyn Clock = &SystemClock;
        let t = c.now();
        assert_eq!(t.offset(), UtcOffset::UTC);
        assert!(t.year() >= 2026);
    }
}
