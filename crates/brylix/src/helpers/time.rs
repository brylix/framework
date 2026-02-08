//! Timestamp helpers for working with datetime fields.
//!
//! Provides utilities for getting the current UTC time and a trait
//! for models that have timestamp fields (created_at, updated_at).
//!
//! # Usage
//!
//! ```rust
//! use brylix::helpers::time::utc_now;
//!
//! let now = utc_now();
//! println!("Current UTC time: {}", now);
//! ```

use chrono::{DateTime, Utc};

/// Get the current UTC timestamp.
///
/// # Example
///
/// ```rust
/// use brylix::helpers::time::utc_now;
///
/// let now = utc_now();
/// ```
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

/// Trait for models with timestamp fields (created_at, updated_at).
///
/// Implement this on your SeaORM ActiveModel to standardize
/// timestamp management across entities.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::helpers::time::{Timestamped, utc_now};
/// use sea_orm::Set;
///
/// impl Timestamped for users::ActiveModel {
///     fn set_created_at(&mut self) {
///         self.created_at = Set(utc_now());
///     }
///
///     fn set_updated_at(&mut self) {
///         self.updated_at = Set(utc_now());
///     }
/// }
///
/// // Usage:
/// let mut model = users::ActiveModel { .. };
/// model.set_timestamps(); // Sets both created_at and updated_at
/// ```
pub trait Timestamped {
    /// Set the created_at field to the current UTC time.
    fn set_created_at(&mut self);

    /// Set the updated_at field to the current UTC time.
    fn set_updated_at(&mut self);

    /// Set both created_at and updated_at to the current UTC time.
    fn set_timestamps(&mut self) {
        self.set_created_at();
        self.set_updated_at();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utc_now() {
        let now = utc_now();
        // Verify it returns a reasonable timestamp (after 2024)
        assert!(now.timestamp() > 1_700_000_000);
    }

    struct MockModel {
        created: bool,
        updated: bool,
    }

    impl Timestamped for MockModel {
        fn set_created_at(&mut self) {
            self.created = true;
        }

        fn set_updated_at(&mut self) {
            self.updated = true;
        }
    }

    #[test]
    fn test_set_timestamps() {
        let mut model = MockModel {
            created: false,
            updated: false,
        };
        model.set_timestamps();
        assert!(model.created);
        assert!(model.updated);
    }

    #[test]
    fn test_set_created_at_only() {
        let mut model = MockModel {
            created: false,
            updated: false,
        };
        model.set_created_at();
        assert!(model.created);
        assert!(!model.updated);
    }

    #[test]
    fn test_set_updated_at_only() {
        let mut model = MockModel {
            created: false,
            updated: false,
        };
        model.set_updated_at();
        assert!(!model.created);
        assert!(model.updated);
    }
}
