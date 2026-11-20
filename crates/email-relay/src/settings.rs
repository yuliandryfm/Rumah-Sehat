//! Pengaturan relay (ADR-026 butir 4). Divalidasi saat start; nilai tidak valid menggagalkan start.

use crate::address::Mailbox;
use std::time::Duration;

/// Panjang maksimum baris perintah SMTP termasuk CRLF (RFC 5321 §4.5.3.1.4).
pub const MAX_COMMAND_LINE: usize = 512;
/// Panjang maksimum baris teks pesan termasuk CRLF (RFC 5321 §4.5.3.1.6; juga batas SES Raw [SRC-580]).
pub const MAX_TEXT_LINE: usize = 1000;
/// Batas atas ukuran pesan yang dapat dikonfigurasi (byte).
pub const MAX_MESSAGE_BYTES_LIMIT: usize = 4 * 1024 * 1024;

/// Pengaturan relay yang sudah divalidasi.
#[derive(Debug, Clone)]
pub struct RelaySettings {
    /// Nama host relay di sapaan dan EHLO (`[a-z0-9.-]`), bukan rahasia.
    pub hostname: String,
    /// Daftar izin pengirim (amplop dan header `From`).
    pub allowed_from: Vec<Mailbox>,
    /// Ukuran maksimum pesan (byte, setelah dot-unstuffing, termasuk CRLF).
    pub max_message_bytes: usize,
    /// Batas global pesan per menit yang diteruskan ke SES.
    pub global_per_minute: u32,
    /// Batas pesan per penerima per jam.
    pub per_recipient_per_hour: u32,
    /// Jumlah koneksi bersamaan maksimum.
    pub max_connections: usize,
    /// Batas waktu menganggur per baca/tulis.
    pub idle_timeout: Duration,
    /// Batas waktu satu sesi SMTP.
    pub session_timeout: Duration,
}

/// Galat pengaturan: nama field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSetting(pub &'static str);

impl RelaySettings {
    /// Memeriksa rentang nilai.
    pub fn validate(&self) -> Result<(), InvalidSetting> {
        let host_ok = !self.hostname.is_empty()
            && self.hostname.len() <= 253
            && self
                .hostname
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.');
        if !host_ok {
            return Err(InvalidSetting("hostname"));
        }
        if self.allowed_from.is_empty() {
            return Err(InvalidSetting("allowed_from"));
        }
        if !(1024..=MAX_MESSAGE_BYTES_LIMIT).contains(&self.max_message_bytes) {
            return Err(InvalidSetting("max_message_bytes"));
        }
        if self.global_per_minute == 0 {
            return Err(InvalidSetting("global_per_minute"));
        }
        if self.per_recipient_per_hour == 0 {
            return Err(InvalidSetting("per_recipient_per_hour"));
        }
        if !(1..=1024).contains(&self.max_connections) {
            return Err(InvalidSetting("max_connections"));
        }
        if self.idle_timeout.is_zero() || self.idle_timeout > Duration::from_secs(300) {
            return Err(InvalidSetting("idle_timeout"));
        }
        if self.session_timeout < self.idle_timeout
            || self.session_timeout > Duration::from_secs(3600)
        {
            return Err(InvalidSetting("session_timeout"));
        }
        Ok(())
    }

    /// Alamat kanonik dari daftar izin yang sama dengan `addr` (tanpa membedakan huruf).
    pub fn allowed(&self, addr: &Mailbox) -> Option<&Mailbox> {
        self.allowed_from.iter().find(|a| a.same(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn sample() -> RelaySettings {
        RelaySettings {
            hostname: "email-relay".to_string(),
            allowed_from: vec![Mailbox::parse("noreply@uji.invalid").unwrap()],
            max_message_bytes: 262_144,
            global_per_minute: 60,
            per_recipient_per_hour: 5,
            max_connections: 16,
            idle_timeout: Duration::from_secs(30),
            session_timeout: Duration::from_secs(120),
        }
    }

    #[test]
    fn ranges_are_enforced() {
        assert_eq!(sample().validate(), Ok(()));
        type Mutate = Box<dyn Fn(&mut RelaySettings)>;
        let cases: Vec<(&str, Mutate)> = vec![
            ("hostname", Box::new(|s| s.hostname = "Relay Host".into())),
            ("allowed_from", Box::new(|s| s.allowed_from.clear())),
            ("max_message_bytes", Box::new(|s| s.max_message_bytes = 10)),
            (
                "max_message_bytes",
                Box::new(|s| s.max_message_bytes = MAX_MESSAGE_BYTES_LIMIT + 1),
            ),
            ("global_per_minute", Box::new(|s| s.global_per_minute = 0)),
            (
                "per_recipient_per_hour",
                Box::new(|s| s.per_recipient_per_hour = 0),
            ),
            ("max_connections", Box::new(|s| s.max_connections = 0)),
            (
                "idle_timeout",
                Box::new(|s| s.idle_timeout = Duration::ZERO),
            ),
            (
                "session_timeout",
                Box::new(|s| s.session_timeout = Duration::from_secs(1)),
            ),
        ];
        for (name, f) in cases {
            let mut s = sample();
            f(&mut s);
            assert_eq!(s.validate(), Err(InvalidSetting(name)), "{name}");
        }
    }

    #[test]
    fn allowed_is_case_insensitive_and_returns_canonical() {
        let s = sample();
        let got = s
            .allowed(&Mailbox::parse("NoReply@UJI.invalid").unwrap())
            .unwrap();
        assert_eq!(got.as_str(), "noreply@uji.invalid");
        assert!(
            s.allowed(&Mailbox::parse("lain@uji.invalid").unwrap())
                .is_none()
        );
    }
}
