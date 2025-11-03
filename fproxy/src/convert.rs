
use std::fmt::Debug;
use ::std::{ffi::{c_char, CString}, slice};

use libloading::Library;
use safer_ffi::{layout::ReprC};



/// Trait to link a type to its proxy. </br>
/// Note that a proxy is never `#[repr(C)]`, to pass values safely
/// over the dll boundary, refer to `fproxy::FToC` and `fproxy::FFromC`.
pub trait FAsProxy<'l> {
  type FSelf;
}

impl<T: FReprC> FAsProxy<'_> for &T {
  type FSelf = Self;
}
impl<T: FReprC> FAsProxy<'_> for &mut T {
  type FSelf = Self;
}


/// Trait to convert an arbitrary type to a proxy.
pub trait FProxyFrom<'l, T> {
  fn proxy_from(value: T, lib: &'l Library) -> Self;
}

impl<T: FReprC + Debug> FProxyFrom<'_, T> for T {
  fn proxy_from(value: T, _: &'_ Library) -> Self {
    dbg!(std::any::type_name::<T>());
    dbg!(&value);
    value
  }
}


/// Local trait for tagging local and std types.
pub trait FLocal { }
/// Trait to indicate a type is `#[repr(C)]`
pub trait FReprC: ReprC + FLocal { }

impl<T: FReprC> FReprC for *const T { }
impl<T: FReprC> FReprC for *mut T { }

//impl<T: ReprC + FLocal> FReprC for T { }

macro_rules! impl_f_local {
  ($T:ty) => {
    impl FLocal for $T { }
  };
}

impl<T: FLocal> FLocal for *const T { }
impl<T: FLocal> FLocal for *mut T { }
impl<T: FLocal> FLocal for &T { }
impl<T: FLocal> FLocal for &mut T { }

#[macro_export]
macro_rules! impl_primitive {
  ($T:ty) => {
    
    impl FAsProxy<'_> for $T {
      type FSelf = Self;
    }

    impl_f_local!($T);
    
    impl FReprC for $T { }
    //impl FReprC for &$T { }
    //impl FReprC for &mut $T { }
    //impl FReprC for *const $T { }
    //impl FReprC for *mut $T { }

    /*
    impl FToC for $T {
      type CType = Self;
      fn to_c(self) -> Self {
        self
      }
    }*/
    impl FToC for &$T {
      type CType = *const $T;
      fn to_c(self) -> Self::CType {
        self as *const $T
      }
    }
    impl FToC for &mut $T {
      type CType = *mut $T;
      fn to_c(self) -> Self::CType {
        self as *mut $T
      }
    } 
  
    impl FProxyFrom<'_, *const $T> for &$T {
      fn proxy_from(ptr: *const $T, _: &Library) -> Self {
        unsafe {
          &*ptr
        }
      }
    }

  };
}

impl_primitive!(());
impl_primitive!(usize);
impl_primitive!(i32);



/// Trait to link types to their `#[repr(C)]` equivalents. </br>
/// The information is used convert function arguments to 
/// c compatible datatypes in order to safely pass the dll boundary. </br>
/// Converts a *Rust* type to a *C* type.
pub trait FToC {
  type CType: FReprC;
  fn to_c(self) -> Self::CType;
}

impl<T: FReprC> FToC for T {
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
 

/// Convert pointer to a rust type withing 
/// the dll, convert to its CType.
/// The trait is unsafe because the caller **must**
/// lookup the conversion function in the library (`lib`).
/// The conversion function **must** know what the original 
/// rust type *was* and what the CType *is*
pub unsafe trait FFromPtr {
  fn from_ptr(ptr: *const (), lib: &Library) -> Self;
}


/*
 * +-----------------------------------+
 * |            Primitives             |
 * +-----------------------------------+
 */
mod primitives {
  
  use super::*;
  use safer_ffi::derive_ReprC;

  #[derive_ReprC]
  #[repr(C)]
  pub struct U128 {
    pub(in super) l: u64,
    pub(in super) r: u64,
  }
  impl FLocal for U128 { }
  impl FReprC for U128 { }

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
  impl FToC for &u128 {
    type CType = <u128 as FToC>::CType;
    fn to_c(self) -> Self::CType {
      (*self).to_c()
    }
  }
  impl FFromC for u128 {
    unsafe fn from_c(c_type: Self::CType) -> Self {
      From::from(c_type)
    }
  }
  impl FProxyFrom<'_, U128> for u128 {
    fn proxy_from(value: U128, _: &Library) -> Self {
      Self::from(value)
    }
  }

  impl FAsProxy<'_> for u128 {
    type FSelf = Self;
  }
  impl FAsProxy<'_> for U128 {
    type FSelf = u128;
  }


  #[derive_ReprC]
  #[repr(C)]
  pub struct FStr {
    data: *const u8,
    len: usize,
  }
  impl FLocal for FStr { }
  impl FReprC for FStr { }

  impl From<&str> for FStr {
    fn from(s: &str) -> Self {
      FStr { 
        len: s.len(),
        data: s.as_ptr(), 
      }
    }
  }
  impl From<FStr> for &str {
    fn from(fstr: FStr) -> Self {
      str::from_utf8(
        unsafe { slice::from_raw_parts(fstr.data, fstr.len) }
      )
        .unwrap()
    }
  }

  impl FToC for &str {
    type CType = FStr;
    fn to_c(self) -> Self::CType {
      From::from(self)
    }
  }
  impl FFromC for &str {
    unsafe fn from_c(c_type: Self::CType) -> Self {
      From::from(c_type)
    }
  }

  impl<'l> FAsProxy<'l> for &'l str {
    type FSelf = &'l str;
  }
  impl<'l> FAsProxy<'l> for &'l &str {
    type FSelf = &'l str;
  }

  impl FLocal for *mut c_char { }
  impl FReprC for *mut c_char { }

  impl FToC for String {
    type CType = *mut c_char;
    fn to_c(self) -> Self::CType {
      CString::new(self)
        .unwrap()
        .into_raw()
    }
  }
  impl FFromC for String {
    unsafe fn from_c(c_type: Self::CType) -> Self {
      unsafe {
        CString::from_raw(c_type)
          .to_str()
          .unwrap()
          .to_string()
      }
    }
  }
  impl FProxyFrom<'_, *mut c_char> for String {
    fn proxy_from(value: *mut c_char, _: &'_ Library) -> Self {
      unsafe { 
        Self::from_c(value)
      }
    }
  }

  impl FAsProxy<'_> for String {
    type FSelf = String;
  }

  impl<'l> FToC for &'l String {
    type CType = <String as FToC>::CType;
    fn to_c(self) -> Self::CType {
      self.clone().to_c()
    }
  }
  impl<'l> FAsProxy<'_> for &'l String {
    type FSelf = &'l str;
  }

  impl FProxyFrom<'_, FStr> for &str {
    fn proxy_from(value: FStr, _: &'_ Library) -> Self {
      From::from(value)
    }
  }
  
  impl FToC for &&str {
    type CType = FStr;
    fn to_c(self) -> Self::CType {
      (*self).to_c()
    }
  }



}
pub use primitives::*;


impl<T> FLocal for safer_ffi::Vec<T> { }
impl<T: FReprC> FReprC for safer_ffi::Vec<T> { }



#[cfg(test)]
pub mod proxy {

  use crate::{FFromC, FToC};
  use super::U128;

  #[test]
  fn test_u128() {
    let u = 123u128;
    println!("{u}");
    let u2 = U128::from(u);
    println!("l: {}, r: {}", u2.l, u2.r);
    assert!(u == u128::from(u2));
  }
  #[test]
  fn test_str() {
    let s = "Hello world!";
    let cstr = s.to_c();
    let s2: &str = unsafe { FFromC::from_c(cstr) };
    assert!(s == s2, "s2: {s2}");
  }
  #[test]
  fn test_string() {
    let s = "Hello world!".to_string();
    let cstr = s.clone().to_c();
    let s2: String = unsafe { FFromC::from_c(cstr) };
    assert!(s == s2, "s2: {s2}");
  }

}