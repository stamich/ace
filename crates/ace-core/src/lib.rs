//! Shared domain model and stable contracts for ACE.
//!
//! This crate intentionally contains no concrete compression implementation. It defines the
//! format-facing IDs, configuration, errors, block profiles and physical plans used by the other crates.

mod block;
mod codec;
mod config;
mod error;
mod limits;
mod plan;
mod profile;
mod stats;

pub use block::*;
pub use codec::*;
pub use config::*;
pub use error::*;
pub use limits::*;
pub use plan::*;
pub use profile::*;
pub use stats::*;
