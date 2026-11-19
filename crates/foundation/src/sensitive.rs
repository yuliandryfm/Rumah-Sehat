//! Pembungkus rahasia (ADR-005): password, PIN, OTP, token, nomor kartu/rekening tidak boleh
//! muncul di log, pesan galat, atau audit. Debug/Display selalu tersamar; nilai hanya
//! lewat [`Sensitive::expose`] yang disengaja. Sengaja tanpa `Serialize`.

use std::fmt;

const REDACTED: &str = "[REDACTED]";

/// Nilai rahasia dengan representasi teks tersamar.
#[derive(Clone, Default)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    /// Membungkus nilai.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Membuka nilai secara eksplisit; pemakaian di titik ini harus dapat dipertanggungjawabkan di review.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Mengambil nilai keluar dari pembungkus.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> From<T> for Sensitive<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_do_not_leak() {
        let s = Sensitive::new("rahasia-123456".to_string());
        assert_eq!(format!("{s:?}"), "[REDACTED]");
        assert_eq!(format!("{s}"), "[REDACTED]");
        assert_eq!(format!("{s:#?}"), "[REDACTED]");
        assert!(!format!("{s:>40?}").contains("rahasia"));
    }

    #[test]
    fn nested_in_derived_debug_does_not_leak() {
        #[derive(Debug)]
        #[allow(dead_code)] // field dibaca lewat Debug saja
        struct Login {
            user: String,
            pin: Sensitive<String>,
            otp: Option<Sensitive<u32>>,
        }
        let l = Login {
            user: "budi".into(),
            pin: Sensitive::new("987654".into()),
            otp: Some(Sensitive::new(424242)),
        };
        let out = format!("{l:?} {l:#?}");
        assert!(out.contains("budi"));
        assert!(!out.contains("987654"));
        assert!(!out.contains("424242"));
    }

    #[test]
    fn expose_and_into_inner_return_value() {
        let s: Sensitive<String> = "token".to_string().into();
        assert_eq!(s.expose(), "token");
        assert_eq!(s.into_inner(), "token");
    }
}
