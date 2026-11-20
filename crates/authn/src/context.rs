//! Konteks autentikasi hasil validasi token.

use crate::realm::Realm;
use std::fmt;
use time::OffsetDateTime;

/// Identitas pemanggil yang **sudah divalidasi** (tanda tangan, issuer, audience, masa berlaku).
///
/// Hanya dapat dibangun oleh [`crate::Authenticator`]: field privat dan tanpa konstruktor publik,
/// sehingga keberadaan nilai ini membuktikan token sudah diverifikasi (dasar KEPUTUSAN-128: cakupan
/// tenant dibangun crate `authz` dari konteks ini, bukan dari isi permintaan).
///
/// Peran dan status akun **tidak** diambil dari token: Keycloak hanya menyimpan identitas minimal,
/// peran disimpan di database aplikasi (ADR-011 butir 6).
#[derive(Clone, PartialEq, Eq)]
pub struct AuthContext {
    realm: Realm,
    subject: String,
    session_id: Option<String>,
    client_id: Option<String>,
    auth_time: Option<OffsetDateTime>,
    expires_at: OffsetDateTime,
}

impl AuthContext {
    pub(crate) fn new(
        realm: Realm,
        subject: String,
        session_id: Option<String>,
        client_id: Option<String>,
        auth_time: Option<OffsetDateTime>,
        expires_at: OffsetDateTime,
    ) -> Self {
        Self {
            realm,
            subject,
            session_id,
            client_id,
            auth_time,
            expires_at,
        }
    }

    /// Realm penerbit token.
    pub fn realm(&self) -> Realm {
        self.realm
    }

    /// Subjek Keycloak (`sub`), dipetakan ke `account.idp_subject` bersama realm.
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// ID sesi Keycloak (`sid`), dipetakan ke `session_revocation.idp_session_id` (F1-T33). Wajib ada
    /// untuk realm akun manusia; `None` hanya untuk realm `integrasi`. Nilai ini tidak boleh ditulis
    /// ke log (ADR-010 butir 5: nilai sesi).
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Klien OIDC penerima token (`azp`).
    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// Waktu autentikasi terakhir (`auth_time`), bila Keycloak menyertakannya (ADR-011 butir 9).
    pub fn auth_time(&self) -> Option<OffsetDateTime> {
        self.auth_time
    }

    /// Batas berlaku token (`exp`).
    pub fn expires_at(&self) -> OffsetDateTime {
        self.expires_at
    }
}

impl fmt::Debug for AuthContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthContext")
            .field("realm", &self.realm)
            .field("subject", &self.subject)
            .field(
                "session_id",
                &self.session_id.as_ref().map(|_| "[REDACTED]"),
            )
            .field("client_id", &self.client_id)
            .field("auth_time", &self.auth_time)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_session_id() {
        let c = AuthContext::new(
            Realm::Pasien,
            "sub-1".into(),
            Some("sesi-rahasia".into()),
            None,
            None,
            OffsetDateTime::UNIX_EPOCH,
        );
        let s = format!("{c:?}");
        assert!(!s.contains("sesi-rahasia"));
        assert!(s.contains("[REDACTED]"));
    }
}
