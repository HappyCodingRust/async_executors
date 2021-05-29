#![cfg_attr(nightly, feature(external_doc, doc_cfg))]
#![cfg_attr(nightly, doc(include = "../README.md"))]
#![doc = ""] // empty doc line to handle missing doc warning when the feature is missing.
#![doc(html_root_url = "https://docs.rs/async_executors")]
// #![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(
    anonymous_parameters,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // unreachable_pub,  // due to https://github.com/rust-lang/rust/issues/64762
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

mod core;
mod runtime;

pub use crate::core::*;
pub use runtime::*;
