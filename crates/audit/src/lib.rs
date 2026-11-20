//! Pustaka audit trail tunggal Ruang Sehat dan RME minimalis (ADR-005, ADR-009, modul kritis T1).
//!
//! Isi (F1-T22): kode event PRD §7.1 ([`EventCode`], E-01–E-18), field PRD §7.2 ([`AuditEvent`]),
//! daftar izin per jenis objek dan masking PRD §7.3 ([`ObjectPolicy`], [`Value`]), serta payload
//! `audit_outbox` (DD A19.1) yang dibentuk [`AuditEvent::finish`] (murni, tanpa I/O).
//! F1-T23: [`in_audited_tx`] memegang begin/commit transaksi tingkat atas, menjalankan aksi bisnis, lalu
//! menulis event-nya ke `audit_outbox` di transaksi yang sama; gagal tulis membatalkan semuanya
//! (ADR-009 butir 2, KEPUTUSAN-15). Tidak ada API publik untuk menulis ke transaksi milik pemanggil.
//! F1-T38: [`insert_failed_access`] menulis event penolakan akses ([`FailedAccessRecord`], hanya E-05
//! hasil gagal tanpa perubahan data) di transaksi tersendiri secara idempoten (`ON CONFLICT DO
//! NOTHING`, satu-satunya constraint unik `audit_outbox` adalah PK `id`), untuk percobaan ulang di luar jalur permintaan; event aksi bisnis tetap lewat
//! [`in_audited_tx`] (KEPUTUSAN-283).
//! F1-T24: [`relay`] memindahkan event dari `audit_outbox` ke database audit (rantai hash lewat crate
//! `audit-store`) dengan satu relay aktif (kunci advisory `platform_jobs::lock`), ADR-009 butir 3.
//!
//! Yang ditegakkan pustaka (ADR-009 butir 7, coding-standards §4.4 butir 5), dibuktikan test di
//! `tests/forbidden_fields.rs`, `tests/outbox_pg*.rs`, dan unit test modul:
//! - Tidak ada parameter untuk password, PIN, OTP, token, rahasia MFA, atau kode pemulihan; nama field
//!   yang memuatnya ditolak di kebijakan objek apa pun.
//! - NIK, nomor kartu, dan nomor rekening yang dinyatakan lewat field `Last4` hanya masuk sebagai
//!   [`Value::Digits`] (dibungkus `Sensitive`) dan hanya keluar sebagai 4 digit terakhir.
//! - Di field lain angka dibatasi: [`Value::Int`] ≤ 6 digit ([`INT_MAX_ABS`]), [`Value::Idr`] ≤ 12 digit
//!   ([`IDR_MAX_ABS`]), versi objek ≤ 6 digit, [`Code`] ≤ 4 digit seluruhnya, versi aplikasi ≤ 8 digit,
//!   teks `Text` paling banyak 3 angka (deret ≥ 4 disamarkan, sisanya disamarkan bila ≥ 4).
//! - Objek klinis/dokumen hanya mencatat nama field yang berubah dan versinya.
//! - Teks bebas hanya di field `Text` yang terdaftar dan user agent, disaring pola terlarang
//!   (`observability::redact::scrub_text`) dan dipotong.
//!
//! Batas yang diketahui (bukan jaminan): nomor rekening 10–12 digit tidak dapat dibedakan dari nominal
//! [`Value::Idr`] yang sah; user agent (dari klien peminta) hanya menyamarkan deret angka > 5 digit, jadi
//! nomor yang dipecah kelompok pendek dapat lolos; nama orang dan alamat di teks `Text` tidak dikenali
//! pola, sehingga field `Text` hanya untuk label non-pribadi dan direview di daftar izin.

mod code;
mod error;
mod event;
mod outbox;
mod policy;
pub mod relay;
mod value;

pub use code::{CODE_MAX_DIGITS, CODE_MAX_LEN, Code, EventCode};
pub use error::{AuditError, AuditErrorKind};
pub use event::{
    Actor, ActorType, AuditContext, AuditEvent, AuditRecord, Disclosure, FailedAccessRecord,
    MAX_SUBJECTS, OBJECT_VERSION_MAX, ObjectId, ObjectRef, Outcome, PAYLOAD_VERSION, Source,
    SourceComponent, Subject, SubjectKind, USER_AGENT_MAX_CHARS,
};
pub use outbox::{AuditedTxError, AuditedWork, in_audited_tx, insert_failed_access};
pub use policy::{
    ContentClass, FieldKind, FieldRule, ObjectPolicy, is_forbidden_field_name, is_number_field_name,
};
pub use value::{FieldChange, IDR_MAX_ABS, INT_MAX_ABS, TEXT_MAX_CHARS, Value};
