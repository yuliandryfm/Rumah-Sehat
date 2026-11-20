//! Realm Keycloak dan backend penerima (ADR-011 butir 3 dan 11).

use std::fmt;

/// Realm Keycloak (ADR-011 butir 3). Nama realm adalah nama di Keycloak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Realm {
    /// Pasien.
    Pasien,
    /// Tenaga kesehatan.
    Nakes,
    /// Admin Ruang Sehat dan admin rekanan.
    Admin,
    /// Pengguna RME minimalis.
    Rme,
    /// Klien sistem rekanan (client credentials, ADR-014).
    Integrasi,
    /// HR perusahaan (CR-021, ADR-027).
    Korporat,
}

impl Realm {
    /// Semua realm.
    pub const ALL: [Realm; 6] = [
        Realm::Pasien,
        Realm::Nakes,
        Realm::Admin,
        Realm::Rme,
        Realm::Integrasi,
        Realm::Korporat,
    ];

    /// Nama realm di Keycloak.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pasien => "pasien",
            Self::Nakes => "nakes",
            Self::Admin => "admin",
            Self::Rme => "rme",
            Self::Integrasi => "integrasi",
            Self::Korporat => "korporat",
        }
    }

    /// Realm berisi akun manusia (token wajib membawa sesi Keycloak `sid`). Realm `integrasi` berisi
    /// klien mesin (client credentials) tanpa sesi pengguna.
    pub const fn is_human(self) -> bool {
        !matches!(self, Self::Integrasi)
    }
}

impl fmt::Display for Realm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Backend yang memvalidasi token. Menegakkan ADR-011 butir 11 di konstruksi konfigurasi: backend
/// Ruang Sehat tidak menerima realm `rme`, dan backend RME hanya menerima realm `rme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Backend Ruang Sehat (`apps/rs-backend`).
    RuangSehat,
    /// Backend RME minimalis (`apps/rme-backend`).
    Rme,
}

impl Backend {
    /// Apakah backend ini boleh menerima token dari `realm`.
    pub const fn accepts(self, realm: Realm) -> bool {
        match self {
            Self::RuangSehat => !matches!(realm, Realm::Rme),
            Self::Rme => matches!(realm, Realm::Rme),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rs_backend_rejects_rme_realm_and_rme_backend_accepts_only_rme() {
        for r in Realm::ALL {
            assert_eq!(Backend::RuangSehat.accepts(r), r != Realm::Rme, "{r}");
            assert_eq!(Backend::Rme.accepts(r), r == Realm::Rme, "{r}");
        }
    }

    #[test]
    fn only_integrasi_is_machine_realm() {
        for r in Realm::ALL {
            assert_eq!(r.is_human(), r != Realm::Integrasi, "{r}");
        }
    }
}
