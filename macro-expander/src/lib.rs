//! A proc macro for debugging proc macros,
//! placing output in a file to give precise errors".
//!
//! See the crate readme for more information.

#![cfg_attr(has_tracked_env_var, feature(proc_macro_tracked_env))]

#[cfg(feature = "enable")]
mod runtime;

#[cfg(feature = "enable")]
pub use macro_expander_macro::debug_expand_macro;

#[cfg(feature = "enable")]
pub use runtime::{debug_expand_simple, debug_expand_simple2, is_macro_debug_enabled};
