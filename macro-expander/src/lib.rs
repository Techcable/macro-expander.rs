//! A proc macro for debugging proc macros,
//! placing output in a file to give precise errors.
//!
//! Currently implemented in terms of the [`expander` crate], where the idea originated from.
//!
//! [`expander` crate]: https://github.com/drahnr/expander
//!
//! # Example
//! Add the following dependency to your proc macro crate.
//! ```toml
//! [dependencies]
//! macro-expander = { version = "0.1", features = ["enable"] }
//! ```
//!
//! Then wrap your macro with `#[debug_expand_macro]`:
//! ```ignore
//! #[proc_macro_derive(Visit)]
//! #[macro_expander::debug_expand_macro]
//! fn derive_visit(input: TokenStream) -> TokenStream {
//!    unimplemented!("your code here")
//! }
//! ```
//!
//! Then if your macro gives errors you can set the environment variable `MACRO_EXPANDER_DEBUG=1`,
//! and errors will point to a specific location in a temporary build file rather than to the macro invocation.
//!
//! If your macro gives a [`compile_error!`], no file is actually used,
//! as that could make error messages lose span information.

#![deny(missing_docs)]
#![cfg_attr(has_tracked_env_var, feature(proc_macro_tracked_env))]

#[cfg(feature = "enable")]
mod runtime;

#[cfg(feature = "macro")]
pub use macro_expander_macro::debug_expand_macro;

#[cfg(feature = "enable")]
pub use runtime::{debug_expand_simple, debug_expand_simple2, is_macro_debug_enabled};
