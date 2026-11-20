//! Pustaka audit trail tunggal Ruang Sehat dan RME minimalis (ADR-005, ADR-009, modul kritis T1).
//!
//! Isi (F1-T22): kode event PRD §7.1 ([`EventCode`], E-01–E-18), field PRD §7.2 ([`AuditEvent`]),
//! daftar izin per jenis objek dan masking PRD §7.3 ([`ObjectPolicy`], [`Value`]), serta payload
//! `audit_outbox` (DD A19.1) yang dibentuk [`AuditEvent::finish`].
//!
//! Larangan yang dijamin API (ADR-009 butir 7, coding-standards §4.4 butir 5):
//! - Tidak ada parameter untuk password, PIN, OTP, token, rahasia MFA, atau kode pemulihan; nama field
//!   yang memuatnya ditolak di kebijakan objek apa pun.
//! - NIK, nomor kartu, dan nomor rekening hanya masuk sebagai [`Value::Digits`] (dibungkus
//!   `Sensitive`) dan hanya keluar sebagai 4 digit terakhir.
//! - Objek klinis/dokumen hanya mencatat nama field yang berubah dan versinya.
//! - Teks bebas hanya di field `Text` yang terdaftar, disaring pola terlarang dan dipotong; nama aksi,
//!   jenis objek, peran, kode alasan, dan ID objek non-UUID berupa [`Code`] tertutup.

mod code;
mod error;
mod event;
mod policy;
mod value;

pub use code::{CODE_MAX_DIGIT_RUN, CODE_MAX_LEN, Code, EventCode};
pub use error::{AuditError, AuditErrorKind};
pub use event::{
    Actor, ActorType, AuditContext, AuditEvent, AuditRecord, Disclosure, MAX_SUBJECTS, ObjectId,
    ObjectRef, Outcome, PAYLOAD_VERSION, Source, SourceComponent, Subject, SubjectKind,
    USER_AGENT_MAX_CHARS,
};
pub use policy::{
    ContentClass, FieldKind, FieldRule, ObjectPolicy, is_forbidden_field_name, is_number_field_name,
};
pub use value::{FieldChange, TEXT_MAX_CHARS, Value};
