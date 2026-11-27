//! Adaptor envelope ADR-023 ke axum. Crate `envelope` tidak bergantung pada framework HTTP
//! (adaptor ada di lapisan aplikasi), sehingga tipe pembungkus ada di sini.
//!
//! Status HTTP selalu diturunkan dari `response_code` (envelope), jadi header dan tiga digit pertama
//! kode tidak mungkin berbeda. Respons galat memakai `application/problem+json`, sukses
//! `application/json` (ADR-023 butir 3).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode, header::CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use envelope::CommonError;
use envelope::{ApiError, CONTENT_TYPE_JSON, CONTENT_TYPE_PROBLEM, Success};
use foundation::CorrelationId;
use serde::Serialize;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

fn json_response<B: Serialize>(
    status: StatusCode,
    content_type: &'static str,
    body: &B,
) -> Response {
    match serde_json::to_vec(body) {
        Ok(bytes) => (
            status,
            [(CONTENT_TYPE, HeaderValue::from_static(content_type))],
            bytes,
        )
            .into_response(),
        // Isi envelope hanya berisi tipe yang selalu dapat diserialisasi; cabang ini pertahanan
        // terakhir dan sengaja tanpa isi supaya tidak membocorkan teks galat (ADR-023 butir 5).
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Respons sukses envelope.
#[derive(Debug)]
pub struct ApiSuccess<T>(pub Success<T>);

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        json_response(self.0.http_status(), CONTENT_TYPE_JSON, &self.0)
    }
}

/// Respons galat envelope dari galat domain yang mengimplementasikan [`ApiError`].
#[derive(Debug)]
pub struct ApiFailure<E: ApiError> {
    error: E,
    correlation_id: CorrelationId,
    type_base: String,
}

impl<E: ApiError> ApiFailure<E> {
    /// `type_base` adalah awalan URI jenis masalah dari konfigurasi.
    pub fn new(error: E, correlation_id: CorrelationId, type_base: &str) -> Self {
        Self {
            error,
            correlation_id,
            type_base: type_base.to_string(),
        }
    }
}

impl<E: ApiError> IntoResponse for ApiFailure<E> {
    fn into_response(self) -> Response {
        let body = self
            .error
            .to_error_body(self.correlation_id, &self.type_base);
        json_response(body.http_status(), CONTENT_TYPE_PROBLEM, &body)
    }
}

/// Pembentuk respons galat envelope untuk middleware autentikasi dan otorisasi (`authn`, `authz`).
#[derive(Debug, Clone)]
pub struct EnvelopeRenderer {
    type_base: Arc<str>,
}

impl EnvelopeRenderer {
    /// `type_base` adalah awalan URI jenis masalah dari konfigurasi.
    pub fn new(type_base: &str) -> Self {
        Self {
            type_base: Arc::from(type_base),
        }
    }
}

impl authn::ErrorRenderer for EnvelopeRenderer {
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response {
        ApiFailure::new(error, correlation_id, &self.type_base).into_response()
    }
}

/// Alamat IP klien dari `authz::ClientIp` (dipasang middleware IP klien `app.rs`), untuk sumber event audit
/// (PRD §7.2) dan batas laju per sumber. `None` bila server dijalankan tanpa `ConnectInfo`.
///
/// Nilainya alamat peer TCP, kecuali peer termasuk [`TrustedProxies`] (`TRUSTED_PROXY_CIDRS`): barulah
/// `X-Forwarded-For` dibaca dengan algoritma "paling kanan yang bukan proxy tepercaya" (KEPUTUSAN-400, TD-22).
/// Tanpa proxy tepercaya (bawaan) tidak ada header klien yang dipercaya.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientAddr(pub Option<IpAddr>);

impl<S: Send + Sync> FromRequestParts<S> for ClientAddr {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(parts.extensions.get::<authz::ClientIp>().map(|c| c.0)))
    }
}

/// Nama header yang dibaca dari proxy tepercaya. Caddy 2.11.9 tanpa `trusted_proxies` membuang nilai kiriman
/// klien dan mengisinya dengan alamat peer koneksinya sendiri (SRC-925, SRC-926).
pub const FORWARDED_FOR_HEADER: &str = "x-forwarded-for";
/// Panjang total maksimum nilai `X-Forwarded-For` (semua baris header) yang diproses; lebih panjang = tidak valid
/// (pakai peer TCP). Caddy di depan API mengirim satu alamat (SRC-926), jadi batas ini hanya tercapai bila ada
/// proxy tepercaya yang meneruskan rantai panjang.
pub const MAX_FORWARDED_FOR_BYTES: usize = 1024;
/// Jumlah entri maksimum yang diperiksa dari kanan, termasuk alamat klien: paling banyak 15 entri tepercaya lalu
/// 1 alamat klien. Bila entri ke-16 dari kanan masih tepercaya = tidak valid (KEPUTUSAN-401, diperjelas KEPUTUSAN-404).
pub const MAX_FORWARDED_FOR_HOPS: usize = 16;
/// Jumlah jaringan maksimum di `TRUSTED_PROXY_CIDRS`.
pub const MAX_TRUSTED_PROXY_NETWORKS: usize = 64;

/// Satu jaringan proxy tepercaya (alamat dasar dengan bit host nol, panjang prefiks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IpNetwork {
    base: IpAddr,
    prefix: u8,
}

fn v4_mask(prefix: u8) -> u32 {
    // Prefiks 1..=32 (0 ditolak saat parse); `checked_shl` menjaga dari geser 32 bit.
    u32::MAX.checked_shl(32 - u32::from(prefix)).unwrap_or(0)
}

fn v6_mask(prefix: u8) -> u128 {
    u128::MAX.checked_shl(128 - u32::from(prefix)).unwrap_or(0)
}

impl IpNetwork {
    /// `alamat` atau `alamat/prefiks`. Ditolak: prefiks 0 (mempercayai seluruh internet), prefiks di luar rentang
    /// atau bukan angka desimal murni, bit host tidak nol (ambigu), alamat IPv6 yang memetakan IPv4
    /// (`::ffff:a.b.c.d`, tulis sebagai IPv4), zona IPv6, kurung siku, dan port.
    fn parse(s: &str) -> Option<Self> {
        let (addr, prefix) = match s.split_once('/') {
            Some((a, p)) => (a, Some(p)),
            None => (s, None),
        };
        let base: IpAddr = addr.parse().ok()?;
        if base.to_canonical() != base {
            return None;
        }
        let max = if base.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            None => max,
            Some(p) if !p.is_empty() && p.len() <= 3 && p.bytes().all(|b| b.is_ascii_digit()) => {
                p.parse::<u8>().ok()?
            }
            Some(_) => return None,
        };
        if prefix == 0 || prefix > max {
            return None;
        }
        let network = Self { base, prefix };
        (network.masked(base) == Some(base)).then_some(network)
    }

    /// Alamat `ip` dengan bit host dinolkan memakai prefiks jaringan ini; `None` bila keluarga alamat berbeda.
    fn masked(&self, ip: IpAddr) -> Option<IpAddr> {
        match (self.base, ip) {
            (IpAddr::V4(_), IpAddr::V4(a)) => Some(IpAddr::V4(Ipv4Addr::from(
                u32::from(a) & v4_mask(self.prefix),
            ))),
            (IpAddr::V6(_), IpAddr::V6(a)) => Some(IpAddr::V6(Ipv6Addr::from(
                u128::from(a) & v6_mask(self.prefix),
            ))),
            _ => None,
        }
    }

    fn contains(&self, ip: IpAddr) -> bool {
        self.masked(ip) == Some(self.base)
    }
}

/// Daftar proxy tepercaya `TRUSTED_PROXY_CIDRS` (KEPUTUSAN-400, TD-22, ADR-031 Konsekuensi): alamat atau CIDR
/// dipisah koma, misalnya `172.30.0.10/32,10.10.0.0/24`. Tidak diisi (bawaan) = tidak ada proxy tepercaya:
/// alamat klien selalu peer TCP. Nilai tidak valid menggagalkan start (fail-closed). `Debug` hanya mencetak jumlah.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct TrustedProxies {
    networks: Vec<IpNetwork>,
}

impl std::fmt::Debug for TrustedProxies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrustedProxies({} jaringan)", self.networks.len())
    }
}

impl std::str::FromStr for TrustedProxies {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let networks = s
            .split(',')
            .map(|item| IpNetwork::parse(item.trim_matches([' ', '\t'])))
            .collect::<Option<Vec<_>>>()
            .ok_or(())?;
        if networks.len() > MAX_TRUSTED_PROXY_NETWORKS {
            return Err(());
        }
        Ok(Self { networks })
    }
}

impl TrustedProxies {
    /// Tanpa proxy tepercaya (alamat klien = peer TCP).
    pub fn none() -> Self {
        Self::default()
    }

    /// Jumlah jaringan tepercaya.
    pub fn len(&self) -> usize {
        self.networks.len()
    }

    /// Tidak ada proxy tepercaya.
    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    fn is_trusted(&self, ip: IpAddr) -> bool {
        self.networks.iter().any(|n| n.contains(ip))
    }

    /// Alamat klien untuk permintaan dari `peer` dengan baris header `X-Forwarded-For` `forwarded_for`
    /// (urutan kemunculan).
    ///
    /// - Peer bukan proxy tepercaya: `peer`; header diabaikan seluruhnya.
    /// - Peer tepercaya: entri dibaca dari kanan; entri pertama yang bukan proxy tepercaya adalah klien. Bila
    ///   semua entri tepercaya, entri paling kiri. Entri di kiri alamat terpilih tidak pernah dibaca, sehingga
    ///   nilai palsu kiriman klien di kiri tidak berpengaruh.
    /// - Header tidak ada, bukan teks ASCII, lebih dari [`MAX_FORWARDED_FOR_BYTES`], entri yang perlu dibaca
    ///   bukan alamat IP polos (port, kurung siku, zona, kosong), atau alamat tak tepercaya tidak ditemukan dalam
    ///   [`MAX_FORWARDED_FOR_HOPS`] entri dari kanan (lebih dari 15 entri tepercaya berturut-turut): `peer`.
    ///
    /// Alamat IPv6 yang memetakan IPv4 dinormalkan ke IPv4, supaya satu klien tidak punya dua kunci batas laju.
    pub fn client_ip<'a>(
        &self,
        peer: IpAddr,
        forwarded_for: impl Iterator<Item = &'a HeaderValue>,
    ) -> IpAddr {
        let peer = peer.to_canonical();
        if !self.is_trusted(peer) {
            return peer;
        }
        self.rightmost_untrusted(forwarded_for).unwrap_or(peer)
    }

    fn rightmost_untrusted<'a>(
        &self,
        forwarded_for: impl Iterator<Item = &'a HeaderValue>,
    ) -> Option<IpAddr> {
        let mut lines = Vec::new();
        // Panjang gabungan seperti satu baris (pemisah ", " antarbaris).
        let mut total = 0usize;
        for v in forwarded_for {
            let line = v.to_str().ok()?;
            if !lines.is_empty() {
                total = total.saturating_add(2);
            }
            total = total.saturating_add(line.len());
            if total > MAX_FORWARDED_FOR_BYTES {
                return None;
            }
            lines.push(line);
        }
        let mut examined = 0usize;
        let mut leftmost_trusted = None;
        for line in lines.iter().rev() {
            for item in line.rsplit(',') {
                examined += 1;
                if examined > MAX_FORWARDED_FOR_HOPS {
                    return None;
                }
                let ip = item
                    .trim_matches([' ', '\t'])
                    .parse::<IpAddr>()
                    .ok()?
                    .to_canonical();
                if !self.is_trusted(ip) {
                    return Some(ip);
                }
                leftmost_trusted = Some(ip);
            }
        }
        leftmost_trusted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn proxies(s: &str) -> TrustedProxies {
        TrustedProxies::from_str(s).unwrap()
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn resolve(p: &TrustedProxies, peer: &str, lines: &[&str]) -> IpAddr {
        let values: Vec<HeaderValue> = lines
            .iter()
            .map(|l| HeaderValue::from_str(l).unwrap())
            .collect();
        p.client_ip(ip(peer), values.iter())
    }

    const CADDY: &str = "172.30.0.10";

    #[test]
    fn trusted_proxy_list_parses_addresses_and_cidrs() {
        let p = proxies("172.30.0.10, 10.10.0.0/24,\tfd00:1::/64");
        assert_eq!(p.len(), 3);
        assert!(p.is_trusted(ip("172.30.0.10")));
        assert!(!p.is_trusted(ip("172.30.0.11")));
        assert!(p.is_trusted(ip("10.10.0.255")));
        assert!(!p.is_trusted(ip("10.10.1.0")));
        assert!(p.is_trusted(ip("fd00:1::abcd")));
        assert!(!p.is_trusted(ip("fd00:2::1")));
        assert!(!p.is_trusted(ip("::ffff:10.10.0.1")), "keluarga berbeda");
        assert!(proxies("192.0.2.1/32").is_trusted(ip("192.0.2.1")));
        assert!(proxies("128.0.0.0/1").is_trusted(ip("200.1.2.3")));
        assert!(proxies("2001:db8::1/128").is_trusted(ip("2001:db8::1")));
        assert_eq!(format!("{p:?}"), "TrustedProxies(3 jaringan)");
        assert!(TrustedProxies::none().is_empty());
    }

    // trace: TD-22, KEPUTUSAN-400 (nilai tidak valid = gagal start; fail-closed)
    #[test]
    fn trusted_proxy_list_rejects_invalid_values() {
        let too_many = (0..=MAX_TRUSTED_PROXY_NETWORKS)
            .map(|n| format!("10.0.{}.{}", n / 256, n % 256))
            .collect::<Vec<_>>()
            .join(",");
        for bad in [
            "",
            " ",
            "10.0.0.0/8,",
            ",10.0.0.0/8",
            "10.0.0.0/8,,10.1.0.0/16",
            "proxy.internal",
            "10.0.0.1/8",
            "10.0.0.0/33",
            "10.0.0.0/0",
            "0.0.0.0/0",
            "::/0",
            "10.0.0.0/+8",
            "10.0.0.0/8x",
            "10.0.0.0/",
            "10.0.0.0/0008",
            "10.0.0.0/8/8",
            "2001:db8::/129",
            "2001:db8::1/64",
            "::ffff:10.0.0.1",
            "fe80::1%eth0",
            "[2001:db8::1]",
            "10.0.0.1:80",
            "10.0.0.256",
            too_many.as_str(),
        ] {
            assert!(TrustedProxies::from_str(bad).is_err(), "{bad:?}");
        }
    }

    // trace: TD-22 (peer tak tepercaya: X-Forwarded-For tidak pernah dipercaya)
    #[test]
    fn untrusted_peer_ignores_forwarded_for() {
        let p = proxies(CADDY);
        assert_eq!(
            resolve(&p, "203.0.113.9", &["198.51.100.1"]),
            ip("203.0.113.9")
        );
        assert_eq!(
            resolve(&TrustedProxies::none(), CADDY, &["198.51.100.1"]),
            ip(CADDY)
        );
    }

    #[test]
    fn trusted_peer_uses_single_forwarded_address() {
        let p = proxies(CADDY);
        assert_eq!(resolve(&p, CADDY, &["198.51.100.1"]), ip("198.51.100.1"));
        assert_eq!(resolve(&p, CADDY, &[" 198.51.100.1\t"]), ip("198.51.100.1"));
        // Tanpa header: peer.
        assert_eq!(resolve(&p, CADDY, &[]), ip(CADDY));
    }

    // trace: TD-22, ADR-031 Konsekuensi (paling kanan yang bukan proxy tepercaya; palsu di kiri diabaikan)
    #[test]
    fn trusted_peer_takes_rightmost_untrusted_address_in_chain() {
        let p = proxies("172.30.0.10,10.10.0.0/24");
        // Klien memalsukan entri kiri; LB tepercaya menambah alamat klien asli, Caddy menambah alamat LB.
        assert_eq!(
            resolve(&p, CADDY, &["1.1.1.1, 198.51.100.7, 10.10.0.5"]),
            ip("198.51.100.7")
        );
        // Sampah di kiri alamat terpilih tidak pernah dibaca.
        assert_eq!(
            resolve(&p, CADDY, &["bukan-ip, [::1]:80, 198.51.100.7"]),
            ip("198.51.100.7")
        );
        // Beberapa baris header digabung sesuai urutan kemunculan.
        assert_eq!(
            resolve(&p, CADDY, &["1.1.1.1", "198.51.100.8, 10.10.0.6"]),
            ip("198.51.100.8")
        );
        // Semua entri tepercaya: entri paling kiri.
        assert_eq!(
            resolve(&p, CADDY, &["10.10.0.7, 10.10.0.5"]),
            ip("10.10.0.7")
        );
    }

    #[test]
    fn trusted_peer_handles_ipv6_and_mapped_addresses() {
        let p = proxies("fd00:1::/64");
        assert_eq!(
            resolve(&p, "fd00:1::10", &["2001:db8::42"]),
            ip("2001:db8::42")
        );
        assert_eq!(
            resolve(&p, "fd00:1::10", &["2001:db8::42, fd00:1::5"]),
            ip("2001:db8::42")
        );
        // Peer IPv4 yang dipetakan ke IPv6 (soket dual-stack) dicocokkan sebagai IPv4.
        let v4 = proxies(CADDY);
        assert_eq!(
            resolve(&v4, "::ffff:172.30.0.10", &["198.51.100.1"]),
            ip("198.51.100.1")
        );
        assert_eq!(
            resolve(&v4, CADDY, &["::ffff:198.51.100.1"]),
            ip("198.51.100.1")
        );
        assert_eq!(
            resolve(&v4, "::ffff:203.0.113.9", &["198.51.100.1"]),
            ip("203.0.113.9")
        );
    }

    // trace: TD-22 (header tidak valid = peer TCP)
    #[test]
    fn trusted_peer_with_invalid_forwarded_for_falls_back_to_peer() {
        let p = proxies(CADDY);
        for bad in [
            "",
            " , ",
            "bukan-ip",
            "198.51.100.1:443",
            "[2001:db8::1]",
            "fe80::1%eth0",
            "198.51.100.1;proto=https",
            "198.51.100.1, ",
            "198.51.100.300",
            "for=198.51.100.1",
        ] {
            assert_eq!(resolve(&p, CADDY, &[bad]), ip(CADDY), "{bad:?}");
        }
        // Bukan teks ASCII.
        let raw = HeaderValue::from_bytes(b"198.51.100.1\xff").unwrap();
        assert_eq!(p.client_ip(ip(CADDY), [raw].iter()), ip(CADDY));
    }

    #[test]
    fn trusted_peer_with_oversized_forwarded_for_falls_back_to_peer() {
        let p = proxies("172.30.0.0/24");
        let addr = "198.51.100.1";
        let long = format!("{}{addr}", " ".repeat(MAX_FORWARDED_FOR_BYTES));
        assert_eq!(resolve(&p, CADDY, &[&long]), ip(CADDY));
        // Tepat di batas masih dibaca.
        let edge = format!("{}{addr}", " ".repeat(MAX_FORWARDED_FOR_BYTES - addr.len()));
        assert_eq!(resolve(&p, CADDY, &[&edge]), ip(addr));
        // Batas berlaku untuk gabungan beberapa baris.
        let half = " ".repeat(MAX_FORWARDED_FOR_BYTES / 2);
        assert_eq!(
            resolve(&p, CADDY, &[&half, &format!("{half}{addr}")]),
            ip(CADDY)
        );
        // Terlalu banyak entri tepercaya berturut-turut dari kanan.
        let hops = vec!["172.30.0.1"; MAX_FORWARDED_FOR_HOPS].join(", ");
        assert_eq!(resolve(&p, CADDY, &[&format!("{addr}, {hops}")]), ip(CADDY));
        let ok = vec!["172.30.0.1"; MAX_FORWARDED_FOR_HOPS - 1].join(", ");
        assert_eq!(resolve(&p, CADDY, &[&format!("{addr}, {ok}")]), ip(addr));
    }
}
