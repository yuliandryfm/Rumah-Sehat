//! Pembuat ID (ADR-002): UUIDv7 dibuat aplikasi, bukan default kolom.

pub use uuid::Uuid;

/// Membuat UUIDv7 baru (urut waktu).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_version_7() {
        assert_eq!(new_id().get_version_num(), 7);
    }

    #[test]
    fn new_id_is_unique() {
        assert_ne!(new_id(), new_id());
    }

    #[test]
    fn new_id_timestamp_prefix_does_not_decrease() {
        let a = new_id();
        let b = new_id();
        // 48 bit pertama adalah cap waktu milidetik.
        assert!(a.as_bytes().get(..6) <= b.as_bytes().get(..6));
    }
}
