//! Satu sesi SMTP (subset RFC 5321 + STARTTLS RFC 3207), ADR-026 butir 2-4.
//!
//! Alur: sapaan 220 → EHLO → STARTTLS (wajib) → TLS → EHLO lagi → MAIL FROM (daftar izin) → RCPT TO
//! (satu penerima) → DATA (CRLF ketat, batas baris dan ukuran) → pemeriksaan header → batas laju → SES
//! secara sinkron → 250 hanya setelah SES menerima.
//!
//! Teks balasan tetap per kelas galat ditambah `ref=<ID korelasi>`; tidak pernah menggemakan perintah,
//! header, badan, atau alamat (M-35). Log hanya memuat kelas hasil, ukuran, ID korelasi, dan `MessageId` SES.

use crate::address::Mailbox;
use crate::io::{LineError, SmtpStream};
use crate::limits::{Decision, RateLimiter, Reservation};
use crate::message::check_headers;
use crate::sender::{MailSender, OutboundMessage, SendOutcome};
use crate::settings::{MAX_COMMAND_LINE, MAX_TEXT_LINE, RelaySettings};
use foundation::CorrelationId;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

/// Jumlah balasan galat sebelum sesi ditutup.
const MAX_ERRORS: u32 = 5;
/// Jumlah perintah maksimum per sesi.
const MAX_COMMANDS: u32 = 100;

/// Bagian bersama semua sesi.
pub(crate) struct Shared<S> {
    pub(crate) settings: RelaySettings,
    pub(crate) limiter: RateLimiter,
    pub(crate) sender: S,
    pub(crate) tls: TlsAcceptor,
}

/// Akhir satu fase sesi.
enum Phase {
    /// Klien meminta STARTTLS dan balasan 220 sudah dikirim.
    Upgrade,
    /// Sesi selesai (QUIT, galat, atau koneksi putus).
    Close,
}

/// Perintah yang dikenali (huruf besar).
fn verb_and_arg(line: &[u8]) -> Option<(String, String)> {
    if line.is_empty() || !line.iter().all(|&b| (0x20..0x7f).contains(&b)) {
        return None;
    }
    let text = std::str::from_utf8(line).ok()?;
    let (verb, arg) = match text.split_once(' ') {
        Some((v, a)) => (v, a.trim()),
        None => (text, ""),
    };
    Some((verb.to_ascii_uppercase(), arg.to_string()))
}

fn reply(code: &str, text: &str) -> Vec<u8> {
    format!("{code} {text}\r\n").into_bytes()
}

fn reply_ref(code: &str, text: &str, corr: &CorrelationId) -> Vec<u8> {
    format!("{code} {text} ref={corr}\r\n").into_bytes()
}

/// Galat sesi: balasan + log kelas, tanpa nilai.
async fn fail<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut SmtpStream<S>,
    errors: &mut u32,
    code: &str,
    text: &str,
    class: &'static str,
) -> Result<(), LineError> {
    let corr = CorrelationId::new();
    tracing::info!(correlation_id = %corr, class = class, smtp_code = code, "relay_command_rejected");
    *errors += 1;
    stream.write_all(&reply_ref(code, text, &corr)).await?;
    if *errors >= MAX_ERRORS {
        stream
            .write_all(&reply("421", "4.7.0 Too many errors, closing connection"))
            .await?;
        return Err(LineError::Io);
    }
    Ok(())
}

/// Menutup sesi karena pembingkaian baris rusak atau batas waktu (tanpa sinkronisasi ulang).
async fn close_on_line_error<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut SmtpStream<S>,
    e: LineError,
) {
    let corr = CorrelationId::new();
    let (code, text, class) = match e {
        LineError::TooLong => (
            "500",
            "5.5.2 Line too long, closing connection",
            "line_too_long",
        ),
        LineError::BadLineEnding => (
            "500",
            "5.5.2 Bad line ending, closing connection",
            "bad_line_ending",
        ),
        LineError::Timeout => ("421", "4.4.2 Timeout, closing connection", "timeout"),
        LineError::Eof | LineError::Io => return,
    };
    tracing::info!(correlation_id = %corr, class = class, "relay_session_closed");
    let _ = stream.write_all(&reply_ref(code, text, &corr)).await;
}

fn ehlo_reply(settings: &RelaySettings, offer_starttls: bool) -> Vec<u8> {
    let mut out = format!("250-{}\r\n", settings.hostname);
    if offer_starttls {
        out.push_str(&format!(
            "250-SIZE {}\r\n250 STARTTLS\r\n",
            settings.max_message_bytes
        ));
    } else {
        out.push_str(&format!("250 SIZE {}\r\n", settings.max_message_bytes));
    }
    out.into_bytes()
}

/// Menangani satu koneksi TCP sampai selesai.
pub(crate) async fn handle<S: MailSender>(tcp: TcpStream, shared: Arc<Shared<S>>) {
    let idle = shared.settings.idle_timeout;
    let mut plain = SmtpStream::new(tcp, idle);
    let greeting = format!("220 {} ESMTP ready\r\n", shared.settings.hostname);
    if plain.write_all(greeting.as_bytes()).await.is_err() {
        return;
    }
    let mut commands = 0u32;
    match plain_phase(&mut plain, &shared.settings, &mut commands).await {
        Phase::Close => return,
        Phase::Upgrade => {}
    }
    if plain.has_buffered() {
        // Data dikirim berbarengan dengan STARTTLS (injeksi perintah sebelum TLS; RFC 3207 §4: STARTTLS wajib
        // perintah terakhir dalam satu kelompok [SRC-587]): tutup. Data yang tiba sesudahnya gagal sebagai TLS.
        let corr = CorrelationId::new();
        tracing::info!(correlation_id = %corr, class = "starttls_pipelining", "relay_session_closed");
        return;
    }
    let tls = match tokio::time::timeout(idle, shared.tls.accept(plain.into_inner())).await {
        Ok(Ok(s)) => s,
        _ => {
            let corr = CorrelationId::new();
            tracing::info!(correlation_id = %corr, class = "tls_handshake_failed", "relay_session_closed");
            return;
        }
    };
    let mut secure = SmtpStream::new(tls, idle);
    secure_phase(&mut secure, &shared, &mut commands).await;
}

/// Sebelum TLS: hanya EHLO/HELO, STARTTLS, NOOP, RSET, QUIT.
async fn plain_phase<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut SmtpStream<S>,
    settings: &RelaySettings,
    commands: &mut u32,
) -> Phase {
    let mut errors = 0u32;
    let mut ehlo = false;
    loop {
        let line = match stream.read_line(MAX_COMMAND_LINE).await {
            Ok(l) => l,
            Err(e) => {
                close_on_line_error(stream, e).await;
                return Phase::Close;
            }
        };
        *commands += 1;
        if *commands > MAX_COMMANDS {
            let _ = stream
                .write_all(&reply("421", "4.7.0 Too many commands, closing connection"))
                .await;
            return Phase::Close;
        }
        let r = match verb_and_arg(&line) {
            None => fail(stream, &mut errors, "500", "5.5.2 Syntax error", "syntax").await,
            Some((verb, arg)) => match verb.as_str() {
                "EHLO" | "HELO" if arg.is_empty() => {
                    fail(
                        stream,
                        &mut errors,
                        "501",
                        "5.5.4 Syntax error in parameters",
                        "syntax",
                    )
                    .await
                }
                "EHLO" => {
                    ehlo = true;
                    stream.write_all(&ehlo_reply(settings, true)).await
                }
                "HELO" => {
                    ehlo = false;
                    stream.write_all(&reply("250", &settings.hostname)).await
                }
                "STARTTLS" if !arg.is_empty() => {
                    fail(
                        stream,
                        &mut errors,
                        "501",
                        "5.5.4 Syntax error in parameters",
                        "syntax",
                    )
                    .await
                }
                "STARTTLS" if !ehlo => {
                    fail(
                        stream,
                        &mut errors,
                        "503",
                        "5.5.1 Bad sequence of commands",
                        "sequence",
                    )
                    .await
                }
                "STARTTLS" => {
                    return match stream
                        .write_all(&reply("220", "2.0.0 Ready to start TLS"))
                        .await
                    {
                        Ok(()) => Phase::Upgrade,
                        Err(_) => Phase::Close,
                    };
                }
                "NOOP" | "RSET" => stream.write_all(&reply("250", "2.0.0 OK")).await,
                "QUIT" => {
                    let _ = stream.write_all(&reply("221", "2.0.0 Bye")).await;
                    return Phase::Close;
                }
                "MAIL" | "RCPT" | "DATA" => {
                    fail(
                        stream,
                        &mut errors,
                        "530",
                        "5.7.0 Must issue a STARTTLS command first",
                        "starttls_required",
                    )
                    .await
                }
                "AUTH" | "VRFY" | "EXPN" | "BDAT" | "ETRN" | "TURN" | "HELP" => {
                    fail(
                        stream,
                        &mut errors,
                        "502",
                        "5.5.1 Command not implemented",
                        "not_implemented",
                    )
                    .await
                }
                _ => {
                    fail(
                        stream,
                        &mut errors,
                        "500",
                        "5.5.2 Command not recognized",
                        "unknown_command",
                    )
                    .await
                }
            },
        };
        if r.is_err() {
            return Phase::Close;
        }
    }
}

/// Transaksi surat dalam sesi TLS.
#[derive(Default)]
struct Txn {
    from: Option<Mailbox>,
    rcpt: Option<Mailbox>,
}

/// `<addr>` setelah `FROM:`/`TO:` (tanpa membedakan huruf), lalu parameter opsional.
fn path_arg<'a>(arg: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let head = arg.get(..keyword.len())?;
    if !head.eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = arg.get(keyword.len()..)?.trim_start();
    let rest = rest.strip_prefix('<')?;
    let close = rest.find('>')?;
    Some((rest.get(..close)?, rest.get(close + 1..)?.trim()))
}

/// Parameter MAIL yang diterima: `SIZE=n` (≤ batas) dan `BODY=7BIT|8BITMIME`.
fn mail_params_ok(params: &str, max: usize) -> Result<(), &'static str> {
    for p in params.split_ascii_whitespace() {
        let (k, v) = p.split_once('=').unwrap_or((p, ""));
        match k.to_ascii_uppercase().as_str() {
            "SIZE" => match v.parse::<usize>() {
                Ok(n) if n <= max => {}
                Ok(_) => return Err("size"),
                Err(_) => return Err("param"),
            },
            "BODY" if v.eq_ignore_ascii_case("7BIT") || v.eq_ignore_ascii_case("8BITMIME") => {}
            _ => return Err("param"),
        }
    }
    Ok(())
}

async fn secure_phase<S: AsyncRead + AsyncWrite + Unpin, M: MailSender>(
    stream: &mut SmtpStream<S>,
    shared: &Shared<M>,
    commands: &mut u32,
) {
    let settings = &shared.settings;
    let mut errors = 0u32;
    let mut ehlo = false;
    let mut txn = Txn::default();
    loop {
        let line = match stream.read_line(MAX_COMMAND_LINE).await {
            Ok(l) => l,
            Err(e) => {
                close_on_line_error(stream, e).await;
                return;
            }
        };
        *commands += 1;
        if *commands > MAX_COMMANDS {
            let _ = stream
                .write_all(&reply("421", "4.7.0 Too many commands, closing connection"))
                .await;
            return;
        }
        let r = match verb_and_arg(&line) {
            None => fail(stream, &mut errors, "500", "5.5.2 Syntax error", "syntax").await,
            Some((verb, arg)) => match verb.as_str() {
                "EHLO" | "HELO" if arg.is_empty() => {
                    fail(
                        stream,
                        &mut errors,
                        "501",
                        "5.5.4 Syntax error in parameters",
                        "syntax",
                    )
                    .await
                }
                "EHLO" => {
                    ehlo = true;
                    txn = Txn::default();
                    stream.write_all(&ehlo_reply(settings, false)).await
                }
                "HELO" => {
                    ehlo = true;
                    txn = Txn::default();
                    stream.write_all(&reply("250", &settings.hostname)).await
                }
                "STARTTLS" => {
                    fail(
                        stream,
                        &mut errors,
                        "503",
                        "5.5.1 TLS already active",
                        "sequence",
                    )
                    .await
                }
                "NOOP" => stream.write_all(&reply("250", "2.0.0 OK")).await,
                "RSET" => {
                    txn = Txn::default();
                    stream.write_all(&reply("250", "2.0.0 OK")).await
                }
                "QUIT" => {
                    let _ = stream.write_all(&reply("221", "2.0.0 Bye")).await;
                    return;
                }
                "MAIL" if !ehlo || txn.from.is_some() => {
                    fail(
                        stream,
                        &mut errors,
                        "503",
                        "5.5.1 Bad sequence of commands",
                        "sequence",
                    )
                    .await
                }
                "MAIL" => match path_arg(&arg, "FROM:") {
                    None => {
                        fail(
                            stream,
                            &mut errors,
                            "501",
                            "5.5.4 Syntax error in parameters",
                            "syntax",
                        )
                        .await
                    }
                    Some((addr, params)) => match (
                        Mailbox::parse(addr),
                        mail_params_ok(params, settings.max_message_bytes),
                    ) {
                        (_, Err("size")) => {
                            fail(
                                stream,
                                &mut errors,
                                "552",
                                "5.3.4 Message size exceeds fixed limit",
                                "size",
                            )
                            .await
                        }
                        (_, Err(_)) => {
                            fail(
                                stream,
                                &mut errors,
                                "555",
                                "5.5.4 Parameter not recognized",
                                "param",
                            )
                            .await
                        }
                        (None, _) => {
                            fail(
                                stream,
                                &mut errors,
                                "501",
                                "5.1.7 Bad sender address syntax",
                                "sender_syntax",
                            )
                            .await
                        }
                        (Some(m), Ok(())) => match settings.allowed(&m) {
                            None => {
                                fail(
                                    stream,
                                    &mut errors,
                                    "550",
                                    "5.7.1 Sender not permitted",
                                    "from_not_allowed",
                                )
                                .await
                            }
                            Some(canonical) => {
                                txn.from = Some(canonical.clone());
                                stream.write_all(&reply("250", "2.1.0 OK")).await
                            }
                        },
                    },
                },
                "RCPT" if txn.from.is_none() => {
                    fail(
                        stream,
                        &mut errors,
                        "503",
                        "5.5.1 Bad sequence of commands",
                        "sequence",
                    )
                    .await
                }
                "RCPT" if txn.rcpt.is_some() => {
                    fail(
                        stream,
                        &mut errors,
                        "452",
                        "4.5.3 Too many recipients",
                        "too_many_recipients",
                    )
                    .await
                }
                "RCPT" => match path_arg(&arg, "TO:") {
                    Some((addr, "")) => match Mailbox::parse(addr) {
                        Some(m) => {
                            txn.rcpt = Some(m);
                            stream.write_all(&reply("250", "2.1.5 OK")).await
                        }
                        None => {
                            fail(
                                stream,
                                &mut errors,
                                "501",
                                "5.1.3 Bad recipient address syntax",
                                "rcpt_syntax",
                            )
                            .await
                        }
                    },
                    Some(_) => {
                        fail(
                            stream,
                            &mut errors,
                            "555",
                            "5.5.4 Parameter not recognized",
                            "param",
                        )
                        .await
                    }
                    None => {
                        fail(
                            stream,
                            &mut errors,
                            "501",
                            "5.5.4 Syntax error in parameters",
                            "syntax",
                        )
                        .await
                    }
                },
                "DATA" if !arg.is_empty() => {
                    fail(
                        stream,
                        &mut errors,
                        "501",
                        "5.5.4 Syntax error in parameters",
                        "syntax",
                    )
                    .await
                }
                "DATA" => {
                    let (Some(from), Some(rcpt)) = (txn.from.take(), txn.rcpt.take()) else {
                        txn = Txn::default();
                        if fail(
                            stream,
                            &mut errors,
                            "503",
                            "5.5.1 Bad sequence of commands",
                            "sequence",
                        )
                        .await
                        .is_err()
                        {
                            return;
                        }
                        continue;
                    };
                    match data(stream, shared, from, rcpt).await {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            close_on_line_error(stream, e).await;
                            return;
                        }
                    }
                }
                "AUTH" | "VRFY" | "EXPN" | "BDAT" | "ETRN" | "TURN" | "HELP" => {
                    fail(
                        stream,
                        &mut errors,
                        "502",
                        "5.5.1 Command not implemented",
                        "not_implemented",
                    )
                    .await
                }
                _ => {
                    fail(
                        stream,
                        &mut errors,
                        "500",
                        "5.5.2 Command not recognized",
                        "unknown_command",
                    )
                    .await
                }
            },
        };
        if r.is_err() {
            return;
        }
    }
}

/// Hasil membaca DATA.
enum Body {
    Complete(Vec<u8>),
    TooLarge,
}

/// Membaca isi DATA sampai `.` dengan dot-unstuffing. Pesan melebihi batas dibaca sampai akhir tanpa
/// disimpan, sampai batas buang 4x ukuran maksimum (di atas itu sesi ditutup).
async fn read_body<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut SmtpStream<S>,
    max: usize,
) -> Result<Body, LineError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut too_large = false;
    let mut discarded = 0usize;
    loop {
        let line = stream.read_line(MAX_TEXT_LINE).await?;
        if line == b"." {
            return Ok(if too_large {
                Body::TooLarge
            } else {
                Body::Complete(buf)
            });
        }
        let content = match line.split_first() {
            Some((b'.', rest)) => rest,
            _ => line.as_slice(),
        };
        if too_large {
            discarded = discarded.saturating_add(content.len() + 2);
            if discarded > max.saturating_mul(4) {
                return Err(LineError::TooLong);
            }
            continue;
        }
        if buf.len() + content.len() + 2 > max {
            too_large = true;
            discarded = buf.len() + content.len() + 2;
            buf = Vec::new();
            continue;
        }
        buf.extend_from_slice(content);
        buf.extend_from_slice(b"\r\n");
    }
}

/// Pesanan kuota per penerima yang dikembalikan saat dibuang, kecuali [`QuotaGuard::keep`] dipanggil (SES
/// menerima pesan). Pengembalian juga terjadi bila sesi dibatalkan di tengah pengiriman (batas waktu sesi).
struct QuotaGuard<'a> {
    limiter: &'a RateLimiter,
    reservation: Option<Reservation>,
}

impl QuotaGuard<'_> {
    fn keep(&mut self) {
        self.reservation = None;
    }
}

impl Drop for QuotaGuard<'_> {
    fn drop(&mut self) {
        if let Some(r) = self.reservation.take() {
            self.limiter.refund(r);
        }
    }
}

/// DATA sampai balasan akhir.
async fn data<S: AsyncRead + AsyncWrite + Unpin, M: MailSender>(
    stream: &mut SmtpStream<S>,
    shared: &Shared<M>,
    from: Mailbox,
    rcpt: Mailbox,
) -> Result<(), LineError> {
    stream
        .write_all(&reply("354", "Start mail input; end with <CRLF>.<CRLF>"))
        .await?;
    let body = read_body(stream, shared.settings.max_message_bytes).await?;
    let corr = CorrelationId::new();
    let raw = match body {
        Body::TooLarge => {
            tracing::info!(correlation_id = %corr, outcome = "rejected", class = "size", "relay_message");
            return stream
                .write_all(&reply_ref(
                    "552",
                    "5.3.4 Message size exceeds fixed limit",
                    &corr,
                ))
                .await;
        }
        Body::Complete(raw) => raw,
    };
    let size = raw.len();
    if let Err(v) = check_headers(&raw, &from, &rcpt) {
        tracing::info!(correlation_id = %corr, outcome = "rejected", class = v.as_str(), size = size, "relay_message");
        return stream
            .write_all(&reply_ref("554", "5.6.0 Message content rejected", &corr))
            .await;
    }
    let mut quota = match shared.limiter.check(&rcpt) {
        Decision::Allow(r) => QuotaGuard {
            limiter: &shared.limiter,
            reservation: Some(r),
        },
        Decision::DropRecipient => {
            // ADR-026 butir 4: 250 dan buang, supaya tidak ada sinyal berbeda bagi pemanggil. Setiap pembuangan
            // juga masuk security log (dasar peringatan; review keamanan F1-T36).
            tracing::warn!(correlation_id = %corr, outcome = "dropped", class = "recipient_rate", size = size, "relay_message");
            observability::security::email_recipient_rate_dropped(corr.as_uuid());
            return stream.write_all(&reply_ref("250", "2.0.0 OK", &corr)).await;
        }
        Decision::Defer => {
            tracing::warn!(correlation_id = %corr, outcome = "deferred", class = "global_rate", size = size, "relay_message");
            return stream
                .write_all(&reply_ref(
                    "451",
                    "4.7.1 Rate limit exceeded, try again later",
                    &corr,
                ))
                .await;
        }
    };
    let msg = OutboundMessage {
        from: from.as_str().to_string(),
        to: rcpt.as_str().to_string(),
        raw,
    };
    match shared.sender.send(&msg).await {
        SendOutcome::Accepted(id) => {
            // Hanya pesan yang diterima SES memakai kuota per penerima; hasil lain mengembalikannya (QuotaGuard).
            quota.keep();
            tracing::info!(correlation_id = %corr, outcome = "sent", size = size, ses_message_id = id.as_str(), "relay_message");
            stream.write_all(&reply_ref("250", "2.0.0 OK", &corr)).await
        }
        SendOutcome::Temporary { kind, status } => {
            tracing::warn!(correlation_id = %corr, outcome = "failed_temporary", class = kind, ses_status = status.unwrap_or(0), size = size, "relay_message");
            stream
                .write_all(&reply_ref(
                    "451",
                    "4.4.1 Temporary delivery failure, try again later",
                    &corr,
                ))
                .await
        }
        SendOutcome::Permanent { kind, status } => {
            tracing::warn!(correlation_id = %corr, outcome = "failed_permanent", class = kind, ses_status = status.unwrap_or(0), size = size, "relay_message");
            stream
                .write_all(&reply_ref(
                    "554",
                    "5.0.0 Message rejected by provider",
                    &corr,
                ))
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbs_are_case_insensitive_and_reject_controls() {
        assert_eq!(verb_and_arg(b"ehlo x"), Some(("EHLO".into(), "x".into())));
        assert_eq!(verb_and_arg(b"QUIT"), Some(("QUIT".into(), String::new())));
        assert_eq!(verb_and_arg(b""), None);
        assert_eq!(verb_and_arg(b"EHLO \x00"), None);
        assert_eq!(verb_and_arg("EHLO \u{e9}".as_bytes()), None);
    }

    #[test]
    fn path_argument_and_mail_params() {
        assert_eq!(path_arg("FROM:<a@b.c>", "FROM:"), Some(("a@b.c", "")));
        assert_eq!(
            path_arg("from: <a@b.c> SIZE=10", "FROM:"),
            Some(("a@b.c", "SIZE=10"))
        );
        assert_eq!(path_arg("TO:a@b.c", "TO:"), None);
        assert_eq!(path_arg("FROM:<a@b.c", "FROM:"), None);
        assert_eq!(path_arg("FRO", "FROM:"), None);
        assert_eq!(mail_params_ok("", 100), Ok(()));
        assert_eq!(mail_params_ok("SIZE=100 BODY=8BITMIME", 100), Ok(()));
        assert_eq!(mail_params_ok("SIZE=101", 100), Err("size"));
        assert_eq!(mail_params_ok("SMTPUTF8", 100), Err("param"));
        assert_eq!(mail_params_ok("AUTH=<>", 100), Err("param"));
    }
}
