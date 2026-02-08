//! Soft delete pattern for entities.
//!
//! Provides a trait and common status constants for implementing
//! soft delete instead of permanent deletion.
//!
//! # Usage
//!
//! ```rust,ignore
//! use brylix::helpers::soft_delete::{SoftDeletable, status};
//! use sea_orm::Set;
//!
//! impl SoftDeletable for posts::ActiveModel {
//!     fn mark_deleted(&mut self) {
//!         self.status = Set(status::DELETED.to_string());
//!     }
//!
//!     fn is_deleted(&self) -> bool {
//!         // Check based on your model's status field
//!         false
//!     }
//! }
//! ```

/// Trait for entities that support soft deletion.
///
/// Instead of permanently deleting records, implementors mark them
/// with a deleted status that can be filtered in queries.
pub trait SoftDeletable {
    /// Mark the entity as deleted.
    fn mark_deleted(&mut self);

    /// Check if the entity is deleted.
    fn is_deleted(&self) -> bool;
}

/// Common status constants for entity lifecycle management.
///
/// # Example
///
/// ```rust
/// use brylix::helpers::soft_delete::status;
///
/// assert_eq!(status::ACTIVE, "active");
/// assert_eq!(status::DELETED, "deleted");
/// ```
pub mod status {
    /// Entity is active and visible
    pub const ACTIVE: &str = "active";

    /// Entity is pending approval or processing
    pub const PENDING: &str = "pending";

    /// Entity has been approved
    pub const APPROVED: &str = "approved";

    /// Entity has been rejected
    pub const REJECTED: &str = "rejected";

    /// Entity has been soft-deleted
    pub const DELETED: &str = "deleted";
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity {
        status: String,
    }

    impl SoftDeletable for MockEntity {
        fn mark_deleted(&mut self) {
            self.status = status::DELETED.to_string();
        }

        fn is_deleted(&self) -> bool {
            self.status == status::DELETED
        }
    }

    #[test]
    fn test_mark_deleted() {
        let mut entity = MockEntity {
            status: status::ACTIVE.to_string(),
        };
        assert!(!entity.is_deleted());

        entity.mark_deleted();
        assert!(entity.is_deleted());
        assert_eq!(entity.status, "deleted");
    }

    #[test]
    fn test_status_constants() {
        assert_eq!(status::ACTIVE, "active");
        assert_eq!(status::PENDING, "pending");
        assert_eq!(status::APPROVED, "approved");
        assert_eq!(status::REJECTED, "rejected");
        assert_eq!(status::DELETED, "deleted");
    }
}
