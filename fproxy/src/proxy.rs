use std::ops::{Deref, DerefMut};

use crate::proxy;



pub trait FInit {
  type In;
  fn init(input: Self::In) -> Self;
}

/// The following macro's are _not intended for direct use_! </br>
/// Instead use `#[derive(FInit)]` or implement `FInit` manually.
#[macro_export]
macro_rules! imp_finit {
  ($T:ty) => {
    fproxy::_imp_finit!($T, fproxy::FInit);
  };
}
/// This macro *must never be used outside of crate fproxy*. </br>
/// It is intended to impl `FInit` on std types.
#[macro_export]
macro_rules! _imp_finit {
  ($T:ty) => {
    _imp_finit!($T, crate::FInit);
  };
  ($T:ty, $path:path) => {
    impl $path for $T {
      type In = ();
      fn init(_: Self::In) -> Self {
        Self::default()
      }
    }
  };
}



pub trait FProxy {
  /// Frees allocated memory in the library. </br>
  /// ### Safety ###
  /// The caller must ensure `self` is dropped after.
  unsafe fn free(&mut self);
}


pub struct FOwned<F: FProxy> {
  proxy: F,
}

pub struct FRef<F: FProxy> {
  proxy: F,
}
pub struct FRefMut<F: FProxy> {
  proxy: F,
}


impl<F> Drop for FOwned<F>
where 
  F: FProxy
{
  fn drop(&mut self) {
    // safety: self is dropped.
    unsafe {
      self.proxy.free();
    }
  }
}

impl<F> Deref for FOwned<F>
where 
  F: FProxy
{
  type Target = F;
  fn deref(&self) -> &Self::Target {
    &self.proxy
  }
}
impl<F> DerefMut for FOwned<F>
where 
  F: FProxy
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.proxy
  }
}

impl<F> From<F> for FOwned<F>
where 
  F: FProxy
{
  fn from(proxy: F) -> Self {
    FOwned { proxy }
  }
}




