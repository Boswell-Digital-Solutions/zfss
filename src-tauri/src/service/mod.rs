//! ZFSS Service Layer
//!
//! Business logic for all canonical objects.

pub mod authority;

pub use authority::{AuthorityAction, require_authority};
