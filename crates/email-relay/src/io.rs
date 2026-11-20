//! Pembaca baris SMTP dengan pemisah CRLF ketat dan batas panjang (ADR-026 butir 4, M-34 (d)).
//!
//! - Baris wajib diakhiri CRLF; LF tanpa CR atau CR yang tidak diikuti LF adalah galat (pembingkaian tidak
//!   dapat dipercaya lagi, sesi ditutup).
//! - Panjang baris (termasuk CRLF) dibatasi; baris lebih panjang adalah galat.
//! - Setiap pembacaan dibatasi waktu menganggur.

use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Galat pembacaan baris.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineError {
    /// Koneksi ditutup sebelum baris lengkap.
    Eof,
    /// Baris melebihi batas.
    TooLong,
    /// LF tanpa CR, atau CR tanpa LF.
    BadLineEnding,
    /// Waktu menganggur habis.
    Timeout,
    /// Galat I/O.
    Io,
}

/// Aliran SMTP dengan penyangga baca.
pub struct SmtpStream<S> {
    inner: S,
    buf: Vec<u8>,
    idle: Duration,
}

impl<S: AsyncRead + AsyncWrite + Unpin> SmtpStream<S> {
    /// Membungkus aliran.
    pub fn new(inner: S, idle: Duration) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(1024),
            idle,
        }
    }

    /// Ada byte yang sudah diterima tetapi belum dibaca (deteksi pipelining setelah STARTTLS).
    pub fn has_buffered(&self) -> bool {
        !self.buf.is_empty()
    }

    /// Melepas aliran dalam. Pemanggil wajib memastikan penyangga kosong lebih dulu.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Menulis balasan (satu atau beberapa baris yang sudah berakhiran CRLF).
    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), LineError> {
        let fut = async {
            self.inner.write_all(data).await?;
            self.inner.flush().await
        };
        match tokio::time::timeout(self.idle, fut).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(LineError::Io),
            Err(_) => Err(LineError::Timeout),
        }
    }

    /// Membaca satu baris tanpa CRLF. `max` adalah panjang maksimum termasuk CRLF.
    pub async fn read_line(&mut self, max: usize) -> Result<Vec<u8>, LineError> {
        let mut scanned = 0usize;
        loop {
            // Cari LF di bagian penyangga yang belum diperiksa.
            if let Some(rel) = self
                .buf
                .get(scanned..)
                .and_then(|b| b.iter().position(|&c| c == b'\n'))
            {
                let lf = scanned + rel;
                if lf + 1 > max {
                    return Err(LineError::TooLong);
                }
                let line: Vec<u8> = self.buf.drain(..=lf).collect();
                let Some((&b'\n', rest)) = line.split_last() else {
                    return Err(LineError::BadLineEnding);
                };
                let Some((&b'\r', content)) = rest.split_last() else {
                    return Err(LineError::BadLineEnding);
                };
                if content.contains(&b'\r') {
                    return Err(LineError::BadLineEnding);
                }
                return Ok(content.to_vec());
            }
            scanned = self.buf.len();
            if scanned >= max {
                return Err(LineError::TooLong);
            }
            if self.buf.contains(&b'\r') && !self.buf.ends_with(b"\r") {
                // CR di tengah tanpa LF sesudahnya.
                return Err(LineError::BadLineEnding);
            }
            let mut chunk = [0u8; 4096];
            let want = chunk.len().min(max.saturating_sub(scanned).max(1));
            let read = tokio::time::timeout(
                self.idle,
                self.inner.read(chunk.get_mut(..want).unwrap_or(&mut [])),
            )
            .await;
            match read {
                Err(_) => return Err(LineError::Timeout),
                Ok(Err(_)) => return Err(LineError::Io),
                Ok(Ok(0)) => return Err(LineError::Eof),
                Ok(Ok(n)) => self.buf.extend_from_slice(chunk.get(..n).unwrap_or(&[])),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn lines(input: &[u8], max: usize) -> Vec<Result<Vec<u8>, LineError>> {
        let (mut a, b) = tokio::io::duplex(64 * 1024);
        a.write_all(input).await.unwrap();
        drop(a);
        let mut s = SmtpStream::new(b, Duration::from_secs(1));
        let mut out = Vec::new();
        loop {
            let r = s.read_line(max).await;
            let stop = r.is_err();
            out.push(r);
            if stop {
                return out;
            }
        }
    }

    #[tokio::test]
    async fn crlf_lines_are_returned_without_terminator() {
        let r = lines(b"EHLO a\r\n\r\nQUIT\r\n", 512).await;
        assert_eq!(
            r,
            vec![
                Ok(b"EHLO a".to_vec()),
                Ok(Vec::new()),
                Ok(b"QUIT".to_vec()),
                Err(LineError::Eof)
            ]
        );
    }

    #[tokio::test]
    async fn bare_lf_and_bare_cr_are_rejected() {
        assert_eq!(
            lines(b"EHLO a\n", 512).await,
            vec![Err(LineError::BadLineEnding)]
        );
        assert_eq!(
            lines(b"EHLO\ra\r\n", 512).await,
            vec![Err(LineError::BadLineEnding)]
        );
        assert_eq!(
            lines(b"EHLO\ra", 512).await,
            vec![Err(LineError::BadLineEnding)]
        );
    }

    #[tokio::test]
    async fn long_lines_are_rejected() {
        let ok = format!("{}\r\n", "a".repeat(510));
        assert_eq!(
            lines(ok.as_bytes(), 512).await.first(),
            Some(&Ok(vec![b'a'; 510]))
        );
        let long = format!("{}\r\n", "a".repeat(511));
        assert_eq!(
            lines(long.as_bytes(), 512).await,
            vec![Err(LineError::TooLong)]
        );
        let endless = "a".repeat(10_000);
        assert_eq!(
            lines(endless.as_bytes(), 512).await,
            vec![Err(LineError::TooLong)]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn idle_read_times_out() {
        let (_a, b) = tokio::io::duplex(1024);
        let mut s = SmtpStream::new(b, Duration::from_secs(5));
        assert_eq!(s.read_line(512).await, Err(LineError::Timeout));
    }
}
