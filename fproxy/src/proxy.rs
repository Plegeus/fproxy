
use std::ops::{Deref, DerefMut};

use safer_ffi::{derive_ReprC, layout::ReprC};


pub trait FProxy {
  /// Frees allocated memory in the library. </br>
  /// ### Safety ###
  /// The caller must ensure `self` is dropped after.
  unsafe fn free(&mut self) {

  }
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


/// Trait to link a type to its proxy. </br>
/// Note that a proxy is never `#[repr(C)]`, to pass values safely
/// over the dll boundary, refer to `fproxy::FToC` and `fproxy::FFromC`.
pub trait FIntoProxy {
  type FSelf<'l>;
}

#[macro_export]
macro_rules! impl_f_into_proxy {
  ($T:ty) => {
    impl_f_into_proxy!(impl crate, $T);
  };
  (impl $crat:tt, $T:ty) => {
    $crat::impl_f_into_proxy!(impl $crat, $T, Self);
  };
  (impl $crat:tt, $T:ty, $U:ty) => {
    impl $crat::FIntoProxy for $T {
      type FSelf<'l> = $crat::FOwned<$U>;
    }
    impl $crat::FIntoProxy for &$T {
      type FSelf<'l> = $crat::FRef<$U>;
    }
    impl $crat::FIntoProxy for &mut $T {
      type FSelf<'l> = $crat::FRefMut<$U>;
    }
  };
}


/// Trait to link types to their `#[repr(C)]` equivalents. </br>
/// The information is used convert function arguments to 
/// c compatible datatypes in order to safely pass the dll boundary. </br>
/// Converts a *Rust* type to a *C* type.
pub trait FToC {
  type CType;
  fn to_c(self) -> Self::CType;
}

#[macro_export]
macro_rules! impl_f_to_c {
  ($T:ty) => {
    crate::impl_f_to_c!(impl crate, $T);
  };
  (impl $crat:tt, $T:ty) => {
    impl $crat::FToC for $T {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        Box::into_raw(Box::new(self)) as *const ()
      }
    }
    impl $crat::FToC for &$T {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        self as *const $T as *const ()
      }
    }
    impl $crat::FToC for &mut $T {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        self as *mut $T as *mut () as *const ()
      }
    }
  };
}


/// Idem `trait FToC`. </br>
/// Converts a *C* type to a *Rust* type.
pub trait FFromC: FToC {
  unsafe fn from_c(c_type: Self::CType) -> Self;
}


#[macro_export]
macro_rules! impl_f_from_c {
  ($T:ty) => {
    crate::impl_f_from_c!(impl crate, $T);
  };
  (impl $crat:tt, $T:ty) => {
    impl $crat::FFromC for $T {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        *Box::from_raw(c_type as *mut $T)
      }
    }
    impl $crat::FFromC for &$T {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        &*(c_type as *const $T)
      }
    }
    impl $crat::FFromC for &mut $T {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        &mut *(c_type as *const $T as *mut $T)
      }
    }
  };
}
 



trait FReprC: ReprC { }
macro_rules! impl_f_repr_c {
  ($T:ty) => {
    impl FReprC for $T { }
  };
}

impl_f_repr_c!(());
impl_f_repr_c!(U128);


impl<T> FProxy for T 
where 
  T: FReprC
{ }
impl<T> FIntoProxy for T 
where 
  T: FReprC
{
  type FSelf<'l> = Self;
}

impl<T> FToC for T 
where 
  T: FReprC
{
  type CType = Self;
  fn to_c(self) -> Self::CType {
    self
  }
}

#[derive_ReprC]
#[repr(C)]
pub struct U128 {
  l: u64,
  r: u64,
}
impl From<u128> for U128 {
  fn from(value: u128) -> Self {
    let r = value & 0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF;
    U128 { 
      l: ((value - r) >> 64) as u64, 
      r: r as u64,
    }
  }
}

impl From<U128> for u128 {
  fn from(value: U128) -> Self {
    ((value.l as u128) << 64) + value.r as u128
  }
}

impl FToC for u128 {
  type CType = U128;
  fn to_c(self) -> Self::CType {
    From::from(self)
  }
}
impl<T> FFromC for T
where 
  T: FToC + From<T::CType>,
{
  unsafe fn from_c(c_type: Self::CType) -> Self {
    Self::from(c_type)
  }
}




#[cfg(test)]
pub mod proxy {

  use crate::proxy::U128;

  #[test]
  fn test_u128() {
    let u = 123u128;
    println!("{u}");
    let u2 = U128::from(u);
    println!("l: {}, r: {}", u2.l, u2.r);
    assert!(u == u128::from(u2));
  }

}

