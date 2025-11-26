//! Core application primitives (engines, orchestrators)

pub mod runtime;
pub mod http;
pub mod bootstrap {
    //! Entry points and high-level initialization hooks.
}

pub use runtime::*;
pub use http::*;
