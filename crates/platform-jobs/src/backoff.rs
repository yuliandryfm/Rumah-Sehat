//! Backoff eksponensial untuk percobaan ulang job: "kembali `queued` dengan `run_at` mundur bertahap"
//! (docs/05-physical-schema.md §9.1, KEPUTUSAN-108). Nilai dasar dan batasnya tidak ditetapkan dokumen
//! mana pun `[TIDAK DIKETAHUI]`, sehingga wajib diberikan pemanggil per jenis pekerja; tidak ada nilai bawaan.
//! Tanpa jitter (tanpa RNG): deterministik dan dapat diuji dengan `FixedClock`.

use crate::error::{JobError, JobErrorKind};
use time::Duration;

/// Jeda sebelum percobaan berikutnya: `base * 2^(attempt-1)`, dibatasi `max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    base: Duration,
    max: Duration,
}

impl Backoff {
    /// `base` positif dan `max >= base`.
    pub fn new(base: Duration, max: Duration) -> Result<Self, JobError> {
        if base.is_positive() && max >= base {
            Ok(Self { base, max })
        } else {
            Err(JobError::new(JobErrorKind::InvalidConfig))
        }
    }

    /// Jeda setelah percobaan ke-`attempt` (dimulai dari 1) gagal. Nilai `attempt < 1` diperlakukan 1.
    pub fn delay_after(&self, attempt: i16) -> Duration {
        let exp = u32::try_from(attempt.max(1) - 1).unwrap_or(0).min(30);
        let factor = 1_i32 << exp;
        match self.base.checked_mul(factor) {
            Some(d) if d <= self.max => d,
            _ => self.max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_doubles_and_is_capped() {
        let b = Backoff::new(Duration::seconds(30), Duration::minutes(10)).unwrap();
        assert_eq!(b.delay_after(1), Duration::seconds(30));
        assert_eq!(b.delay_after(2), Duration::seconds(60));
        assert_eq!(b.delay_after(3), Duration::seconds(120));
        assert_eq!(b.delay_after(5), Duration::seconds(480));
        assert_eq!(b.delay_after(6), Duration::minutes(10));
        assert_eq!(b.delay_after(i16::MAX), Duration::minutes(10));
        assert_eq!(b.delay_after(0), Duration::seconds(30));
        assert_eq!(b.delay_after(-5), Duration::seconds(30));
    }

    #[test]
    fn huge_base_saturates_to_max() {
        let b = Backoff::new(Duration::MAX, Duration::MAX).unwrap();
        assert_eq!(b.delay_after(10), Duration::MAX);
    }

    #[test]
    fn invalid_config_is_rejected() {
        for (base, max) in [
            (Duration::ZERO, Duration::minutes(1)),
            (Duration::seconds(-1), Duration::minutes(1)),
            (Duration::minutes(2), Duration::minutes(1)),
        ] {
            assert_eq!(
                Backoff::new(base, max).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidConfig)
            );
        }
    }
}
