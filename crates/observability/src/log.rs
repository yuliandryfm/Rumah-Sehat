//! Application log dan security log berformat JSON dengan redaksi (ADR-005, ADR-010, KEPUTUSAN-124,
//! coding-standards §4).
//!
//! Satu `tracing` Layer menulis satu baris JSON per kejadian. Format ditulis sendiri (bukan fitur `json`
//! tracing-subscriber) supaya setiap field melewati redaksi nama dan pola ([`crate::redact`]) sebelum
//! keluar, dan supaya field baku tidak dapat dipalsukan dari kode aplikasi:
//! - `timestamp`, `level`, `component`, `log_stream`, `target` ditetapkan layer; field dengan nama itu
//!   dari kejadian atau span dibuang.
//! - `correlation_id` hanya keluar bila berbentuk UUID valid (KEPUTUSAN-123); selain itu dibuang.
//! - Teks selalu di-escape JSON, sehingga sisipan baris baru tidak dapat memalsukan baris log lain.
//! - Aliran security (KEPUTUSAN-124) ditentukan oleh target kejadian ([`crate::security::SECURITY_TARGET`]),
//!   ditulis ke sink terpisah dengan `log_stream = "security"`, dan tidak terpengaruh `LOG_LEVEL`.

use crate::config::LogLevel;
use crate::redact::{REDACTED, is_sensitive_key, scrub_text};
use crate::security::SECURITY_TARGET;
use foundation::{Clock, CorrelationId};
use std::fmt::{self, Write as _};
use std::io::Write as _;
use std::sync::{Arc, Mutex};
use time::format_description::well_known::Rfc3339;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

/// Komponen proses (field `component`, coding-standards §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    /// `rs-backend api`.
    Api,
    /// `rs-backend realtime`.
    Realtime,
    /// `rs-backend worker`.
    Worker,
    /// `audit-relay` Ruang Sehat.
    AuditRelay,
    /// `audit-verify` Ruang Sehat.
    AuditVerify,
    /// `privacy-worker` Ruang Sehat.
    PrivacyWorker,
    /// `rme-backend api`.
    RmeApi,
    /// `rme-backend worker`.
    RmeWorker,
    /// `audit-relay` RME.
    RmeAuditRelay,
    /// `audit-verify` RME.
    RmeAuditVerify,
    /// `privacy-worker` RME.
    RmePrivacyWorker,
}

impl Component {
    /// Nilai teks pada field `component`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Realtime => "realtime",
            Self::Worker => "worker",
            Self::AuditRelay => "audit-relay",
            Self::AuditVerify => "audit-verify",
            Self::PrivacyWorker => "privacy-worker",
            Self::RmeApi => "rme-api",
            Self::RmeWorker => "rme-worker",
            Self::RmeAuditRelay => "rme-audit-relay",
            Self::RmeAuditVerify => "rme-audit-verify",
            Self::RmePrivacyWorker => "rme-privacy-worker",
        }
    }
}

/// Tujuan baris log. Satu panggilan = satu baris lengkap tanpa pemisah baris.
pub trait LogSink: Send + Sync + 'static {
    /// Menulis satu baris.
    fn write_line(&self, line: &str);
}

/// Menulis ke stdout (dikumpulkan agen log kontainer).
#[derive(Debug, Clone, Copy, Default)]
pub struct StdoutSink;

impl LogSink for StdoutSink {
    fn write_line(&self, line: &str) {
        let mut out = std::io::stdout().lock();
        // Kegagalan menulis log tidak boleh menjatuhkan proses.
        let _ = writeln!(out, "{line}");
    }
}

/// Menampung baris di memori; untuk test yang menangkap keluaran log.
#[derive(Debug, Clone, Default)]
pub struct MemorySink(Arc<Mutex<Vec<String>>>);

impl MemorySink {
    /// Salinan semua baris yang sudah ditulis.
    pub fn lines(&self) -> Vec<String> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Semua baris digabung dengan pemisah baris.
    pub fn text(&self) -> String {
        self.lines().join("\n")
    }
}

impl LogSink for MemorySink {
    fn write_line(&self, line: &str) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(line.to_string());
    }
}

#[derive(Debug, Clone)]
enum Val {
    /// Teks yang sudah melewati `scrub_text`.
    Text(String),
    /// Literal JSON siap pakai (angka, boolean, null).
    Raw(String),
    /// Nilai tersamar karena nama field terlarang.
    Redacted,
    /// Teks mentah khusus `correlation_id`; hanya keluar bila lolos validasi UUID.
    UnverifiedCorrelation(String),
}

#[derive(Debug, Clone, Default)]
struct Fields(Vec<(String, Val)>);

impl Fields {
    fn put(&mut self, key: &str, val: Val) {
        if let Some(slot) = self.0.iter_mut().find(|(k, _)| k == key) {
            slot.1 = val;
        } else {
            self.0.push((key.to_string(), val));
        }
    }

    fn text_field(&mut self, field: &Field, raw: &str) {
        let key = field.name();
        if key == "correlation_id" {
            // Hanya UUID valid yang disimpan; nilai tidak valid dibuang dan tidak boleh menimpa
            // ID valid dari span induk (KEPUTUSAN-123).
            if CorrelationId::parse(raw).is_ok() {
                self.put(key, Val::UnverifiedCorrelation(raw.to_string()));
            }
        } else if is_sensitive_key(key) {
            self.put(key, Val::Redacted);
        } else {
            self.put(key, Val::Text(scrub_text(raw)));
        }
    }

    fn raw_field(&mut self, field: &Field, literal: String) {
        let key = field.name();
        if key == "correlation_id" {
            return;
        }
        if is_sensitive_key(key) {
            self.put(key, Val::Redacted);
        } else {
            self.put(key, Val::Raw(literal));
        }
    }
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.text_field(field, &format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.text_field(field, value);
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.text_field(field, &value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.raw_field(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.raw_field(field, value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.raw_field(field, value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.raw_field(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.raw_field(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        let literal = if value.is_finite() {
            value.to_string()
        } else {
            "null".to_string()
        };
        self.raw_field(field, literal);
    }
}

/// Field yang hanya boleh ditetapkan layer.
const RESERVED: &[&str] = &["timestamp", "level", "component", "log_stream", "target"];

fn push_json_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{2028}' || c == '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Layer log JSON. Bangun dengan [`JsonLogLayer::new`], pasang lewat [`install`] atau [`subscriber`].
pub struct JsonLogLayer {
    component: Component,
    max_level: Level,
    clock: Arc<dyn Clock>,
    app: Arc<dyn LogSink>,
    security: Arc<dyn LogSink>,
}

impl JsonLogLayer {
    /// Layer yang menulis ke stdout untuk kedua aliran (agen log merutekan `log_stream`, KEPUTUSAN-124).
    pub fn new(component: Component, level: LogLevel, clock: Arc<dyn Clock>) -> Self {
        Self {
            component,
            max_level: level.as_tracing(),
            clock,
            app: Arc::new(StdoutSink),
            security: Arc::new(StdoutSink),
        }
    }

    /// Mengganti tujuan tulis aliran application dan security.
    pub fn with_sinks(mut self, app: Arc<dyn LogSink>, security: Arc<dyn LogSink>) -> Self {
        self.app = app;
        self.security = security;
        self
    }

    fn render(&self, event: &Event<'_>, mut fields: Fields, is_security: bool) -> String {
        let meta = event.metadata();
        let mut message = None;
        let mut correlation = None;
        let mut rest = Vec::new();
        for (k, v) in fields.0.drain(..) {
            if k == "message" {
                message = Some(v);
            } else if k == "correlation_id" {
                if let Val::UnverifiedCorrelation(raw) = v {
                    correlation = CorrelationId::parse(&raw).ok();
                }
            } else if !RESERVED.contains(&k.as_str()) {
                rest.push((k, v));
            }
        }

        let mut out = String::with_capacity(256);
        out.push_str("{\"timestamp\":");
        push_json_str(
            &mut out,
            &self.clock.now().format(&Rfc3339).unwrap_or_default(),
        );
        out.push_str(",\"level\":");
        push_json_str(&mut out, &meta.level().as_str().to_ascii_lowercase());
        out.push_str(",\"component\":");
        push_json_str(&mut out, self.component.as_str());
        if is_security {
            out.push_str(",\"log_stream\":\"security\"");
        }
        if let Some(c) = correlation {
            out.push_str(",\"correlation_id\":");
            push_json_str(&mut out, &c.to_string());
        }
        if let Some(Val::Text(m)) = message {
            out.push_str(",\"message\":");
            push_json_str(&mut out, &m);
        }
        out.push_str(",\"target\":");
        push_json_str(&mut out, meta.target());
        for (k, v) in rest {
            out.push(',');
            push_json_str(&mut out, &k);
            out.push(':');
            match v {
                Val::Text(t) => push_json_str(&mut out, &t),
                Val::Raw(r) => out.push_str(&r),
                Val::Redacted | Val::UnverifiedCorrelation(_) => push_json_str(&mut out, REDACTED),
            }
        }
        out.push('}');
        out
    }
}

impl<S> Layer<S> for JsonLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(fields);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut ext = span.extensions_mut();
            if let Some(fields) = ext.get_mut::<Fields>() {
                values.record(fields);
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();
        let is_security = meta.target() == SECURITY_TARGET;
        if !is_security && *meta.level() > self.max_level {
            return;
        }

        // Field span dari akar ke dalam, lalu field kejadian menimpa.
        let mut merged = Fields::default();
        if let Some(scope) = ctx.event_scope(event) {
            let spans: Vec<_> = scope.collect();
            for span in spans.into_iter().rev() {
                if let Some(f) = span.extensions().get::<Fields>() {
                    for (k, v) in &f.0 {
                        merged.put(k, v.clone());
                    }
                }
            }
        }
        let mut own = Fields::default();
        event.record(&mut own);
        for (k, v) in own.0 {
            merged.put(&k, v);
        }

        let line = self.render(event, merged, is_security);
        if is_security {
            self.security.write_line(&line);
        } else {
            self.app.write_line(&line);
        }
    }
}

/// Subscriber lengkap berisi layer; untuk `tracing::subscriber::set_default` di test.
pub fn subscriber(layer: JsonLogLayer) -> impl Subscriber + Send + Sync {
    Registry::default().with(layer)
}

/// Galat pemasangan: subscriber global sudah terpasang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallError;

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("subscriber log global sudah terpasang")
    }
}

impl std::error::Error for InstallError {}

/// Memasang layer sebagai subscriber global proses. Dipanggil sekali saat start.
pub fn install(layer: JsonLogLayer) -> Result<(), InstallError> {
    tracing::subscriber::set_global_default(subscriber(layer)).map_err(|_| InstallError)
}
