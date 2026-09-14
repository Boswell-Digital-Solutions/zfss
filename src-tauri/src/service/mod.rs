//! ZFSS Service Layer
//!
//! Business logic for all canonical objects.

pub mod authority;
pub mod error;
pub mod input;

pub use authority::{AuthorityAction, require_authority};
