//! Tipe dasar lintas crate Ruang Sehat (KEPUTUSAN-110, docs/coding-standards.md §1.1).
//!
//! Isi: [`Clock`] (KEPUTUSAN-131), [`new_id`] UUIDv7 (ADR-002), [`Idr`] (KEPUTUSAN-118),
//! [`CorrelationId`] (KEPUTUSAN-123), dan [`Sensitive`] (ADR-005).

mod clock;
mod correlation;
mod id;
mod money;
mod sensitive;

pub use clock::{Clock, FixedClock, SystemClock};
pub use correlation::{CorrelationId, CorrelationIdError};
pub use id::{Uuid, new_id};
pub use money::{Idr, MoneyError};
pub use sensitive::Sensitive;
