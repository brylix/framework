//! Pagination utilities for GraphQL APIs.
//!
//! Provides generic pagination types that work with any entity type,
//! following the connection pattern commonly used in GraphQL APIs.
//!
//! # Usage
//!
//! ```rust
//! use brylix::graphql::pagination::{Connection, PageInfo, page_info};
//!
//! let info = page_info(100, 1, 10);
//! assert!(info.has_next_page);
//! assert!(!info.has_previous_page);
//! assert_eq!(info.total_pages, 10);
//! ```

use async_graphql::SimpleObject;

/// Pagination metadata for a page of results.
#[derive(Debug, Clone, SimpleObject)]
pub struct PageInfo {
    /// Whether there are more pages after the current page
    pub has_next_page: bool,

    /// Whether there are pages before the current page
    pub has_previous_page: bool,

    /// Total number of pages
    pub total_pages: u64,
}

/// Calculate pagination info from total count, page number, and page size.
///
/// # Arguments
///
/// * `total_count` - Total number of items across all pages
/// * `page` - Current page number (1-based)
/// * `per_page` - Number of items per page
///
/// # Example
///
/// ```rust
/// use brylix::graphql::pagination::page_info;
///
/// let info = page_info(95, 2, 10);
/// assert!(info.has_next_page);
/// assert!(info.has_previous_page);
/// assert_eq!(info.total_pages, 10);
/// ```
pub fn page_info(total_count: u64, page: u64, per_page: u64) -> PageInfo {
    let total_pages = if per_page == 0 {
        0
    } else {
        (total_count + per_page - 1) / per_page
    };

    PageInfo {
        has_next_page: page < total_pages,
        has_previous_page: page > 1,
        total_pages,
    }
}

/// A paginated connection wrapper for any item type.
///
/// Use this to return paginated results from GraphQL resolvers.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::graphql::pagination::Connection;
///
/// async fn list_users(ctx: &Context<'_>, page: u64, per_page: u64) -> Result<Connection<User>> {
///     let (items, total_count) = UserService::paginated(db, page, per_page).await?;
///     Ok(Connection::new(items, total_count, page, per_page))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Connection<T> {
    /// The items for the current page
    pub items: Vec<T>,

    /// Total count of items across all pages
    pub total_count: u64,

    /// Pagination metadata
    pub page_info: PageInfo,
}

impl<T> Connection<T> {
    /// Create a new Connection from items, total count, and pagination params.
    pub fn new(items: Vec<T>, total_count: u64, page: u64, per_page: u64) -> Self {
        Self {
            items,
            total_count,
            page_info: page_info(total_count, page, per_page),
        }
    }
}

/// Trait for converting paginated query results into a Connection.
///
/// Implement this for your service return types to enable `.into_connection()`.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::graphql::pagination::IntoConnection;
///
/// // If your service returns (Vec<T>, u64):
/// impl<T> IntoConnection<T> for (Vec<T>, u64) {
///     fn into_connection(self, page: u64, per_page: u64) -> Connection<T> {
///         Connection::new(self.0, self.1, page, per_page)
///     }
/// }
/// ```
pub trait IntoConnection<T> {
    /// Convert into a paginated Connection.
    fn into_connection(self, page: u64, per_page: u64) -> Connection<T>;
}

/// Blanket implementation for (Vec<T>, u64) tuples.
///
/// This allows any `(items, total_count)` tuple to be converted into a Connection.
impl<T> IntoConnection<T> for (Vec<T>, u64) {
    fn into_connection(self, page: u64, per_page: u64) -> Connection<T> {
        Connection::new(self.0, self.1, page, per_page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_info_first_page() {
        let info = page_info(100, 1, 10);
        assert!(info.has_next_page);
        assert!(!info.has_previous_page);
        assert_eq!(info.total_pages, 10);
    }

    #[test]
    fn test_page_info_middle_page() {
        let info = page_info(100, 5, 10);
        assert!(info.has_next_page);
        assert!(info.has_previous_page);
        assert_eq!(info.total_pages, 10);
    }

    #[test]
    fn test_page_info_last_page() {
        let info = page_info(100, 10, 10);
        assert!(!info.has_next_page);
        assert!(info.has_previous_page);
        assert_eq!(info.total_pages, 10);
    }

    #[test]
    fn test_page_info_single_page() {
        let info = page_info(5, 1, 10);
        assert!(!info.has_next_page);
        assert!(!info.has_previous_page);
        assert_eq!(info.total_pages, 1);
    }

    #[test]
    fn test_page_info_empty() {
        let info = page_info(0, 1, 10);
        assert!(!info.has_next_page);
        assert!(!info.has_previous_page);
        assert_eq!(info.total_pages, 0);
    }

    #[test]
    fn test_page_info_zero_per_page() {
        let info = page_info(100, 1, 0);
        assert_eq!(info.total_pages, 0);
    }

    #[test]
    fn test_page_info_partial_last_page() {
        let info = page_info(95, 10, 10);
        assert!(!info.has_next_page);
        assert!(info.has_previous_page);
        assert_eq!(info.total_pages, 10);
    }

    #[test]
    fn test_connection_new() {
        let items = vec![1, 2, 3];
        let conn = Connection::new(items, 30, 1, 10);
        assert_eq!(conn.items.len(), 3);
        assert_eq!(conn.total_count, 30);
        assert!(conn.page_info.has_next_page);
    }

    #[test]
    fn test_into_connection() {
        let result: (Vec<i32>, u64) = (vec![1, 2, 3], 30);
        let conn = result.into_connection(1, 10);
        assert_eq!(conn.items.len(), 3);
        assert_eq!(conn.total_count, 30);
        assert!(conn.page_info.has_next_page);
    }
}
