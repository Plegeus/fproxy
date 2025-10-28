
/// Traits and impl related to generating proxies.
pub mod proxy;
/// Conversions of types from rust to C to proxies and visa versa.
pub mod convert;
/// Module containing mappings for rust's std types.
pub mod std;

pub use fproxy_derive::*;
pub use libloading;
pub use delegate;
pub use proxy::{FAllocated, FProxy, FOwned, FRef, FRefMut};
pub use convert::{FProxyFrom, FAsProxy, FToC, FFromC};
use ::std::{path::Path};
// Users should be able to `use fproxy::iter::...`
pub use std::*;

use libloading::Library;

pub struct FLib(pub Library);
impl FLib {
  /// Create a new library object, it is needed when a proxy
  /// was annotated with `"lib"` as it will take the library as argument.
  /// Panics if the underlying `libloading::Library` constructor 
  /// returns `Err`.
  pub fn new(path: impl AsRef<Path>) -> Self {
    unsafe {
      let mut lib = path.as_ref().to_path_buf();
      #[cfg(target_os = "windows")]
      lib.set_extension("dll");
      #[cfg(target_os = "macos")] {
        let name = lib.file_name().unwrap().to_str().unwrap();
        let name = format!("lib{name}");
        lib.set_file_name(name);
        lib.set_extension("dylib");
      }
      FLib(Library::new(lib.into_os_string().as_os_str()).unwrap())
    }
  }
}





