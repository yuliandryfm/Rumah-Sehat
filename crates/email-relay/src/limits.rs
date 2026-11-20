//! Batas laju di memori (ADR-026 butir 4): global per menit dan per penerima per jam, jendela tetap.
//!
//! Kunci per penerima adalah HMAC-SHA256 alamat (huruf kecil) dengan kunci acak per proses, sehingga peta di
//! memori tidak memuat alamat dan tidak dapat dicocokkan dengan tabel hash yang dihitung di muka.
//! Waktu memakai `tokio::time::Instant` (monoton; dapat dihentikan di test).

use crate::address::Mailbox;
use crypto::{CryptoError, HmacSha256Key, SHA256_LEN, fill_random};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::Instant;

/// Jendela batas global.
pub const GLOBAL_WINDOW: Duration = Duration::from_secs(60);
/// Jendela batas per penerima.
pub const RECIPIENT_WINDOW: Duration = Duration::from_secs(3600);
/// Jumlah maksimum kunci penerima di memori; di atas ini pesan ditunda (sementara), tidak dibuang.
pub const MAX_TRACKED_RECIPIENTS: usize = 50_000;

/// Keputusan batas laju untuk satu pesan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Teruskan ke SES.
    Allow,
    /// Batas per penerima tercapai: balas 250 dan buang (tanpa sinyal berbeda bagi pemanggil).
    DropRecipient,
    /// Batas global tercapai atau pelacak penuh: balasan sementara 4xx.
    Defer,
}

#[derive(Clone, Copy)]
struct Window {
    start: Instant,
    count: u32,
}

impl Window {
    fn hit(&mut self, now: Instant, len: Duration, max: u32) -> bool {
        if now.duration_since(self.start) >= len {
            self.start = now;
            self.count = 0;
        }
        if self.count >= max {
            return false;
        }
        self.count += 1;
        true
    }
}

struct State {
    global: Window,
    recipients: HashMap<[u8; SHA256_LEN], Window>,
}

/// Pembatas laju bersama semua sesi.
pub struct RateLimiter {
    key: HmacSha256Key,
    global_per_minute: u32,
    per_recipient_per_hour: u32,
    state: Mutex<State>,
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RateLimiter(global {}/menit, penerima {}/jam)",
            self.global_per_minute, self.per_recipient_per_hour
        )
    }
}

impl RateLimiter {
    /// Membuat pembatas dengan kunci HMAC acak baru.
    pub fn new(global_per_minute: u32, per_recipient_per_hour: u32) -> Result<Self, CryptoError> {
        let mut seed = [0u8; 32];
        fill_random(&mut seed)?;
        let key = HmacSha256Key::new(&seed);
        seed.fill(0);
        Ok(Self {
            key: key?,
            global_per_minute,
            per_recipient_per_hour,
            state: Mutex::new(State {
                global: Window {
                    start: Instant::now(),
                    count: 0,
                },
                recipients: HashMap::new(),
            }),
        })
    }

    /// Mencatat satu pesan untuk `rcpt` dan memutuskan nasibnya. Batas per penerima diperiksa lebih dulu,
    /// sehingga pesan yang dibuang tidak menghabiskan kuota global (dan kuota SES).
    pub fn check(&self, rcpt: &Mailbox) -> Decision {
        let now = Instant::now();
        let k = self.key.sign(rcpt.key().as_bytes());
        // Mutex beracun (panik di pemegang lain) diperlakukan sebagai "tunda", bukan panik.
        let Ok(mut st) = self.state.lock() else {
            return Decision::Defer;
        };
        if !st.recipients.contains_key(&k) && st.recipients.len() >= MAX_TRACKED_RECIPIENTS {
            st.recipients
                .retain(|_, w| now.duration_since(w.start) < RECIPIENT_WINDOW);
            if st.recipients.len() >= MAX_TRACKED_RECIPIENTS {
                return Decision::Defer;
            }
        }
        let w = st.recipients.entry(k).or_insert(Window {
            start: now,
            count: 0,
        });
        if !w.hit(now, RECIPIENT_WINDOW, self.per_recipient_per_hour) {
            return Decision::DropRecipient;
        }
        let max = self.global_per_minute;
        if !st.global.hit(now, GLOBAL_WINDOW, max) {
            return Decision::Defer;
        }
        Decision::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mb(s: &str) -> Mailbox {
        Mailbox::parse(s).unwrap()
    }

    #[tokio::test(start_paused = true)]
    async fn per_recipient_limit_drops_and_resets_after_window() {
        let l = RateLimiter::new(100, 2).unwrap();
        let a = mb("a@uji.invalid");
        assert_eq!(l.check(&a), Decision::Allow);
        assert_eq!(l.check(&mb("A@UJI.INVALID")), Decision::Allow);
        assert_eq!(l.check(&a), Decision::DropRecipient);
        assert_eq!(l.check(&mb("b@uji.invalid")), Decision::Allow);
        tokio::time::advance(RECIPIENT_WINDOW).await;
        assert_eq!(l.check(&a), Decision::Allow);
    }

    #[tokio::test(start_paused = true)]
    async fn global_limit_defers_and_resets_after_window() {
        let l = RateLimiter::new(2, 100).unwrap();
        assert_eq!(l.check(&mb("a@uji.invalid")), Decision::Allow);
        assert_eq!(l.check(&mb("b@uji.invalid")), Decision::Allow);
        assert_eq!(l.check(&mb("c@uji.invalid")), Decision::Defer);
        tokio::time::advance(GLOBAL_WINDOW).await;
        assert_eq!(l.check(&mb("c@uji.invalid")), Decision::Allow);
    }

    #[test]
    fn debug_has_no_addresses() {
        let l = RateLimiter::new(1, 1).unwrap();
        let _ = l.check(&mb("rahasia@uji.invalid"));
        assert!(!format!("{l:?}").contains("rahasia"));
    }
}
