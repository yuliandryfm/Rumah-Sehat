//! Batas laju dalam proses untuk endpoint publik (F1-T94/F1-T93; ADR-012 butir 3 "batas laju per sumber dan
//! batas laju global", ADR-027 butir 5, M-47 "anggaran batas laju publik terpisah dari API pasien").
//!
//! Jendela tetap 60 detik dengan dua anggaran: global (seluruh sumber) dan per sumber (alamat IP klien,
//! `http_support::ClientAddr`). Penghitung hanya di memori satu proses `api`; batas laju terdistribusi di
//! Valkey (ADR-007, ADR-025) dan batas per IP di Caddy belum terpasang, sehingga batas efektif = batas ini
//! dikali jumlah instans. Alamat klien = peer TCP, atau alamat dari `X-Forwarded-For` bila peer termasuk
//! `TRUSTED_PROXY_CIDRS` (KEPUTUSAN-400). Tanpa daftar itu di balik reverse proxy, "sumber" adalah proxy dan
//! anggaran per sumber praktis menjadi anggaran per proxy/BFF. Kunci IPv6 adalah alamat penuh (TD-55).
//!
//! Kunci sumber bawaan adalah alamat IP; pembatas yang sama dipakai dengan kunci ID akun untuk batas laju per
//! pengguna terautentikasi (F1-T109, KEPUTUSAN-389: pengajuan ulang verifikasi perusahaan per akun HR).
//!
//! Durasi diukur dengan `Instant` (monotonik), bukan jam dinding (coding-standards §5.7 membolehkan
//! `Instant::now` untuk durasi).

use std::collections::HashMap;
use std::hash::Hash;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Panjang jendela.
pub const WINDOW: Duration = Duration::from_secs(60);
/// Batas jumlah sumber yang dilacak per jendela; sumber berikutnya berbagi satu ember luapan, sehingga
/// memori tetap terbatas walau banyak alamat berbeda.
pub const MAX_TRACKED_SOURCES: usize = 50_000;

/// Anggaran per jendela 60 detik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateBudget {
    /// Permintaan per sumber per menit.
    pub per_source: u32,
    /// Permintaan seluruh sumber per menit.
    pub global: u32,
}

#[derive(Debug)]
struct Window<K> {
    started: Instant,
    global: u32,
    sources: HashMap<K, u32>,
    overflow: u32,
}

/// Hasil pemeriksaan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Diteruskan.
    Allowed,
    /// Ditolak (429). `first` = penolakan pertama sumber/global ini di jendela berjalan (untuk security log
    /// sekali per jendela, bukan per permintaan).
    Limited {
        /// Penolakan pertama di jendela ini.
        first: bool,
    },
}

/// Pembatas jendela tetap; aman dibagi antar-thread. `K` = kunci sumber (bawaan: alamat IP peer).
#[derive(Debug)]
pub struct RateLimiter<K = Option<IpAddr>> {
    budget: RateBudget,
    window: Mutex<Window<K>>,
}

impl<K: Eq + Hash> RateLimiter<K> {
    /// Pembatas baru. Anggaran 0 menolak semua permintaan (gagal-tertutup).
    pub fn new(budget: RateBudget) -> Self {
        Self {
            budget,
            window: Mutex::new(Window {
                started: Instant::now(),
                global: 0,
                sources: HashMap::new(),
                overflow: 0,
            }),
        }
    }

    /// Anggaran.
    pub fn budget(&self) -> RateBudget {
        self.budget
    }

    /// Mencatat satu permintaan dari `source` sekarang.
    pub fn check(&self, source: K) -> Verdict {
        self.check_at(source, Instant::now())
    }

    /// Mencatat satu permintaan pada instan `now` (untuk test).
    pub fn check_at(&self, source: K, now: Instant) -> Verdict {
        // Mutex yang teracuni tetap dipakai: isinya hanya penghitung, tidak ada invarian yang rusak.
        let mut w = self.window.lock().unwrap_or_else(|e| e.into_inner());
        if now.saturating_duration_since(w.started) >= WINDOW {
            w.started = now;
            w.global = 0;
            w.sources.clear();
            w.overflow = 0;
        }
        w.global = w.global.saturating_add(1);
        let tracked = w.sources.len() < MAX_TRACKED_SOURCES || w.sources.contains_key(&source);
        let per_source = if tracked {
            let c = w.sources.entry(source).or_insert(0);
            *c = c.saturating_add(1);
            *c
        } else {
            w.overflow = w.overflow.saturating_add(1);
            w.overflow
        };
        let over_global = w.global > self.budget.global;
        let over_source = per_source > self.budget.per_source;
        if over_global || over_source {
            let first = (over_global && w.global == self.budget.global.saturating_add(1))
                || (over_source && per_source == self.budget.per_source.saturating_add(1));
            Verdict::Limited { first }
        } else {
            Verdict::Allowed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ip(n: u8) -> Option<IpAddr> {
        Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, n)))
    }

    #[test]
    fn per_source_budget_resets_after_window() {
        let l = RateLimiter::new(RateBudget {
            per_source: 2,
            global: 100,
        });
        let t = Instant::now();
        assert_eq!(l.check_at(ip(1), t), Verdict::Allowed);
        assert_eq!(l.check_at(ip(1), t), Verdict::Allowed);
        assert_eq!(l.check_at(ip(1), t), Verdict::Limited { first: true });
        assert_eq!(l.check_at(ip(1), t), Verdict::Limited { first: false });
        // Sumber lain tidak terpengaruh.
        assert_eq!(l.check_at(ip(2), t), Verdict::Allowed);
        assert_eq!(l.check_at(ip(1), t + WINDOW), Verdict::Allowed);
    }

    #[test]
    fn global_budget_limits_all_sources() {
        let l = RateLimiter::new(RateBudget {
            per_source: 100,
            global: 3,
        });
        let t = Instant::now();
        for n in 1..=3 {
            assert_eq!(l.check_at(ip(n), t), Verdict::Allowed);
        }
        assert_eq!(l.check_at(ip(4), t), Verdict::Limited { first: true });
        assert_eq!(l.check_at(None, t), Verdict::Limited { first: false });
    }

    #[test]
    fn zero_budget_denies_everything() {
        let l: RateLimiter = RateLimiter::new(RateBudget {
            per_source: 0,
            global: 0,
        });
        assert!(matches!(l.check(None), Verdict::Limited { .. }));
    }

    // trace: F1-T109 KEPUTUSAN-389: kunci ID akun (batas per pengguna terautentikasi) memakai anggaran yang sama.
    #[test]
    fn account_keyed_budget_is_per_account() {
        let l: RateLimiter<u128> = RateLimiter::new(RateBudget {
            per_source: 1,
            global: 100,
        });
        let t = Instant::now();
        assert_eq!(l.check_at(1, t), Verdict::Allowed);
        assert_eq!(l.check_at(1, t), Verdict::Limited { first: true });
        assert_eq!(l.check_at(2, t), Verdict::Allowed);
    }
}
