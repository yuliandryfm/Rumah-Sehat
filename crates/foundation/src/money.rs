//! Uang: bilangan bulat Rupiah (`i64`), tanpa float/desimal (ADR-015, KEPUTUSAN-118).
//! Luapan adalah galat, bukan pembulatan diam-diam (coding-standards §3.4).

use std::fmt;

/// Galat aritmetika uang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyError {
    /// Hasil di luar rentang `i64`.
    Overflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("luapan aritmetika uang"),
        }
    }
}

impl std::error::Error for MoneyError {}

/// Nominal Rupiah utuh. Bertanda supaya entri reversal/koreksi dapat dinyatakan (ADR-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Idr(i64);

impl Idr {
    /// Nol Rupiah.
    pub const ZERO: Idr = Idr(0);

    /// Membungkus nilai Rupiah.
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Nilai mentah (untuk kolom `bigint` dan JSON integer).
    pub const fn get(self) -> i64 {
        self.0
    }

    /// Penjumlahan terperiksa.
    pub fn checked_add(self, rhs: Idr) -> Result<Idr, MoneyError> {
        self.0
            .checked_add(rhs.0)
            .map(Idr)
            .ok_or(MoneyError::Overflow)
    }

    /// Pengurangan terperiksa.
    pub fn checked_sub(self, rhs: Idr) -> Result<Idr, MoneyError> {
        self.0
            .checked_sub(rhs.0)
            .map(Idr)
            .ok_or(MoneyError::Overflow)
    }

    /// Perkalian dengan kuantitas bilangan bulat, terperiksa.
    pub fn checked_mul(self, qty: i64) -> Result<Idr, MoneyError> {
        self.0.checked_mul(qty).map(Idr).ok_or(MoneyError::Overflow)
    }

    /// Negasi terperiksa (`i64::MIN` meluap).
    pub fn checked_neg(self) -> Result<Idr, MoneyError> {
        self.0.checked_neg().map(Idr).ok_or(MoneyError::Overflow)
    }

    /// Jumlah seluruh elemen; galat pada luapan di langkah mana pun.
    pub fn checked_sum<I: IntoIterator<Item = Idr>>(iter: I) -> Result<Idr, MoneyError> {
        iter.into_iter().try_fold(Idr::ZERO, Idr::checked_add)
    }
}

impl fmt::Display for Idr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} IDR", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_mul_basic() {
        let a = Idr::new(250_000);
        assert_eq!(a.checked_add(Idr::new(1)), Ok(Idr::new(250_001)));
        assert_eq!(a.checked_sub(Idr::new(300_000)), Ok(Idr::new(-50_000)));
        assert_eq!(a.checked_mul(3), Ok(Idr::new(750_000)));
        assert_eq!(a.checked_neg(), Ok(Idr::new(-250_000)));
        assert_eq!(Idr::default(), Idr::ZERO);
        assert_eq!(a.get(), 250_000);
    }

    #[test]
    fn add_overflow_is_error() {
        assert_eq!(
            Idr::new(i64::MAX).checked_add(Idr::new(1)),
            Err(MoneyError::Overflow)
        );
        assert_eq!(
            Idr::new(i64::MIN).checked_add(Idr::new(-1)),
            Err(MoneyError::Overflow)
        );
    }

    #[test]
    fn sub_overflow_is_error() {
        assert_eq!(
            Idr::new(i64::MIN).checked_sub(Idr::new(1)),
            Err(MoneyError::Overflow)
        );
        assert_eq!(
            Idr::new(i64::MAX).checked_sub(Idr::new(-1)),
            Err(MoneyError::Overflow)
        );
    }

    #[test]
    fn mul_overflow_is_error() {
        assert_eq!(Idr::new(i64::MAX).checked_mul(2), Err(MoneyError::Overflow));
        assert_eq!(
            Idr::new(i64::MIN).checked_mul(-1),
            Err(MoneyError::Overflow)
        );
        assert_eq!(Idr::new(i64::MAX).checked_mul(1), Ok(Idr::new(i64::MAX)));
    }

    #[test]
    fn neg_min_overflows() {
        assert_eq!(Idr::new(i64::MIN).checked_neg(), Err(MoneyError::Overflow));
    }

    #[test]
    fn sum_ok_and_overflow() {
        let ok = Idr::checked_sum([Idr::new(1), Idr::new(2), Idr::new(3)]);
        assert_eq!(ok, Ok(Idr::new(6)));
        assert_eq!(Idr::checked_sum([]), Ok(Idr::ZERO));
        let bad = Idr::checked_sum([Idr::new(i64::MAX), Idr::new(1), Idr::new(-1)]);
        assert_eq!(bad, Err(MoneyError::Overflow));
    }

    #[test]
    fn ordering_and_display() {
        assert!(Idr::new(1) < Idr::new(2));
        assert_eq!(Idr::new(1500).to_string(), "1500 IDR");
        assert_eq!(MoneyError::Overflow.to_string(), "luapan aritmetika uang");
    }
}
