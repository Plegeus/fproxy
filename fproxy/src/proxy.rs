
use std::ops::{Deref, DerefMut};

use libloading::Library;
use safer_ffi::{derive_ReprC, layout::ReprC};


/// Trait to control proxies from the application. <br/>
/// Everything that is a proxy implements this trait,
/// it is automatically implemented with `#[fproxy::proxy]`.
pub trait FProxy {
  /// Frees allocated memory in the library. </br>
  /// ### Safety ###
  /// The caller must ensure `self` is dropped after.
  unsafe fn free(&mut self) {

  }
}

/// Owned representation of a proxy. <br/>
/// When `Self` is dropped, the proxy will be freed.
pub struct FOwned<F: FProxy> {
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

impl<'l, F, U> FProxyFrom<'l, U> for FOwned<F> 
where 
  F: FProxyFrom<'l, U> + FProxy
{
  fn proxy_from(value: U, lib: &'l Library) -> Self {
    F::proxy_from(value, lib)
      .into()
  }
}

/// A reference to a proxy. <br/>
/// For example: `&MyPlugin` translates to `FRef<FMyPlugin<'l>>`.
#[derive(Clone, Copy)]
pub struct FRef<F: FProxy> {
  proxy: F,
}

impl<F> Deref for FRef<F>
where 
  F: FProxy
{
  type Target = F;
  fn deref(&self) -> &Self::Target {
    &self.proxy
  }
}


impl<F> From<F> for FRef<F>
where 
  F: FProxy
{
  fn from(proxy: F) -> Self {
    FRef { proxy }
  }
}

impl<'l, F, U> FProxyFrom<'l, U> for FRef<F> 
where 
  F: FProxyFrom<'l, U> + FProxy
{
  fn proxy_from(value: U, lib: &'l Library) -> Self {
    F::proxy_from(value, lib)
      .into()
  }
}

/// Like `FRef<F>`, but for mutable references.
pub struct FRefMut<F: FProxy> {
  proxy: F,
}

impl<F> Deref for FRefMut<F>
where 
  F: FProxy
{
  type Target = F;
  fn deref(&self) -> &Self::Target {
    &self.proxy
  }
}
impl<F> DerefMut for FRefMut<F>
where 
  F: FProxy
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.proxy
  }
}


impl<F> From<F> for FRefMut<F>
where 
  F: FProxy
{
  fn from(proxy: F) -> Self {
    FRefMut { proxy }
  }
}

impl<'l, F, U> FProxyFrom<'l, U> for FRefMut<F> 
where 
  F: FProxyFrom<'l, U> + FProxy
{
  fn proxy_from(value: U, lib: &'l Library) -> Self {
    F::proxy_from(value, lib)
      .into()
  }
}



/// Trait to link a type to its proxy. </br>
/// Note that a proxy is never `#[repr(C)]`, to pass values safely
/// over the dll boundary, refer to `fproxy::FToC` and `fproxy::FFromC`.
pub trait FAsProxy<'l> {
  type FSelf;
}
macro_rules! impl_f_as_proxy {
  ($T:ty) => {
    impl FAsProxy<'_> for $T {
      type FSelf = Self;
    }
  };
}

/// Trait to convert an arbitrary type to a proxy.
pub trait FProxyFrom<'l, T> {
  fn proxy_from(value: T, lib: &'l Library) -> Self;
}

macro_rules! impl_f_proxy_from {
  ($T:ty) => {
    impl FProxyFrom<'_, $T> for $T {
      fn proxy_from(value: $T, _: &Library) -> Self {
        value
      }
    }
  };
}

/// Trait to convert an arbitrary type to a proxy.
//pub trait FIntoProxy<'l>: FAsProxy<'l> {
//  fn into_proxy(self, lib: &'l Library) -> Self::FSelf;
//}

//#[macro_export]
//macro_rules! impl_f_into_proxy {
//  ($T:ty) => {
//    impl<'l> FIntoProxy<'l> for $T {
//      fn into_proxy(self, _: &'l crate::libloading::Library) -> Self::FSelf {
//        self
//      }
//    }
//  };
//  //(impl $crat:tt, $T:ty) => {
//  //  $crat::impl_f_into_proxy!(impl $crat, $T, Self);
//  //};
//  //(impl $crat:tt, $T:ty, $U:ty) => {
//  //  impl<'l> $crat::FIntoProxy for $T {
//  //    fn into_proxy(self, _: &'l $crat::libloading::Library) -> Self::FSelf {
//  //      self
//  //    }
//  //  }
//  //  impl<'l> $crat::FIntoProxy for &$T {
//  //    fn into_proxy(self, _: &'l $crat::libloading::Library) -> Self::FSelf {
//  //      self
//  //    }
//  //  }
//  //  impl<'l> $crat::FIntoProxy for &mut $T {
//  //    fn into_proxy(self, _: &'l $crat::libloading::Library) -> Self::FSelf {
//  //      self
//  //    }
//  //  }
//  //};
//}



/// Local trait for tagging local and std types.
trait FLocal { }
/// Trait to indicate a type is `#[repr(C)]`
trait FReprC: ReprC + FLocal { }

impl<T: ReprC + FLocal> FReprC for T { }

macro_rules! impl_f_local {
  ($T:ty) => {
    impl FLocal for $T { }
  };
}

impl<T: FLocal> FLocal for *const T { }
impl<T: FLocal> FLocal for *mut T { }

macro_rules! impl_primitive {
  ($T:ty) => {
    impl_f_as_proxy!($T);
    //impl_f_proxy_from!($T);
    impl_f_local!($T);
  };
}

impl_primitive!(());



/// Trait to link types to their `#[repr(C)]` equivalents. </br>
/// The information is used convert function arguments to 
/// c compatible datatypes in order to safely pass the dll boundary. </br>
/// Converts a *Rust* type to a *C* type.
pub trait FToC {
  type CType: FReprC;
  fn to_c(self) -> Self::CType;
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
 


impl<T> FFromC for T
where 
  T: FToC + From<T::CType>,
{
  unsafe fn from_c(c_type: Self::CType) -> Self {
    Self::from(c_type)
  }
} 
impl<T> FProxyFrom<'_, T::CType> for T 
where 
  T: FToC + From<T::CType>
{
  fn proxy_from(value: T::CType, _: &'_ Library) -> Self {
    Self::from(value)
  }
}


#[derive_ReprC]
#[repr(C)]
pub struct U128 {
  l: u64,
  r: u64,
}
impl FLocal for U128 { }

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

//impl FProxyFrom<'_, U128> for u128 {
//  fn proxy_from(value: U128, _: &'_ Library) -> Self {
//    From::from(value)
//  }
//}


impl FAsProxy<'_> for u128 {
  type FSelf = Self;
}
impl FAsProxy<'_> for U128 {
  type FSelf = u128;
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

