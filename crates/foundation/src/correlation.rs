//! ID korelasi (KEPUTUSAN-123): hanya UUID valid yang diterima dari luar, sehingga teks bebas
//! tidak dapat disisipkan ke log.

use crate::id::{Uuid, new_id};
use std::fmt;

/// Galat penguraian [`CorrelationId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrelationIdError;

impl fmt::Display for CorrelationIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("correlation id bukan UUID yang valid")
    }
}

impl std::error::Error for CorrelationIdError {}

/// Pengenal korelasi satu permintaan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Membuat ID baru (UUIDv7).
    pub fn new() -> Self {
        Self(new_id())
    }

    /// Menguraikan nilai dari luar (mis. header `X-Correlation-Id`); hanya bentuk UUID
    /// berhubung (8-4-4-4-12, 36 karakter) yang diterima.
    pub fn parse(input: &str) -> Result<Self, CorrelationIdError> {
        if input.len() != 36 {
            return Err(CorrelationIdError);
        }
        Uuid::parse_str(input)
            .map(Self)
            .map_err(|_| CorrelationIdError)
    }

    /// UUID di dalamnya.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_v7_and_roundtrips() {
        let c = CorrelationId::new();
        assert_eq!(c.as_uuid().get_version_num(), 7);
        assert_eq!(CorrelationId::parse(&c.to_string()), Ok(c));
    }

    #[test]
    fn parse_accepts_valid_uuid() {
        let s = "0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3c4d";
        assert_eq!(
            CorrelationId::parse(s).map(|c| c.to_string()),
            Ok(s.to_string())
        );
    }

    #[test]
    fn parse_rejects_invalid_and_injection() {
        for bad in [
            "",
            "abc",
            "0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3c4",
            "0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3c4d\n",
            "0193a5c27b1e7c3a9d412f6e8a1b3c4d",
            "{0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3c4d}",
            "0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3cZZ",
            "line1\nlevel=ERROR forged entry          ",
        ] {
            assert_eq!(
                CorrelationId::parse(bad),
                Err(CorrelationIdError),
                "{bad:?}"
            );
        }
    }
}
