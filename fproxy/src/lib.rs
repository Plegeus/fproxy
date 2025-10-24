
/// Traits and impl related to generating proxies.
pub mod proxy;
/// Conversions of types from rust to C to proxies and visa versa.
pub mod convert;
/// Module containing mappings for rust's std types.
pub mod std;

pub use fproxy_derive::*;
pub use libloading;
pub use delegate;
pub use proxy::{FProxy, FOwned, FRef, FRefMut};
pub use convert::{FProxyFrom, FAsProxy, FToC, FFromC};
// Users should be able to `use fproxy::iter::...`
pub use std::*;








