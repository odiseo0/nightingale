use uuid::Uuid;

// Port so tests and offline paths can control identifier generation
// (Master Plan §11.4).
pub trait IdGenerator {
    fn new_id(&self) -> Uuid;
}

/// UUIDv7 canonical identifiers, as required by the Nightingale API contract.
/// Uuid::now_v7 guarantees process-wide creation order.
/// Source: https://docs.rs/uuid/latest/uuid/struct.Uuid.html#method.now_v7
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_ordered_ids() {
        let generator = UuidV7Generator;
        let first = generator.new_id();
        let second = generator.new_id();

        assert_ne!(first, second);
        assert!(second > first);
        assert_eq!(Some(uuid::Version::SortRand), first.get_version());
    }
}
