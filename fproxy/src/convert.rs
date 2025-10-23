
use libloading::Library;
use safer_ffi::{derive_ReprC, layout::ReprC};



/// Trait to link a type to its proxy. </br>
/// Note that a proxy is never `#[repr(C)]`, to pass values safely
/// over the dll boundary, refer to `fproxy::FToC` and `fproxy::FFromC`.
pub trait FAsProxy<'l> {
  type FSelf;
}

impl<T: ReprC> FAsProxy<'_> for &T {
  type FSelf = Self;
}
impl<T: ReprC> FAsProxy<'_> for &mut T {
  type FSelf = Self;
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

/*
macro_rules! impl_f_proxy_from {
  ($T:ty) => {
    impl FProxyFrom<'_, $T> for $T {
      fn proxy_from(value: $T, _: &Library) -> Self {
        value
      }
    }
  };
} */


/// Local trait for tagging local and std types.
pub trait FLocal { }
/// Trait to indicate a type is `#[repr(C)]`
pub trait FReprC: ReprC + FLocal { }

impl<T: ReprC + FLocal> FReprC for T { }

macro_rules! impl_f_local {
  ($T:ty) => {
    impl FLocal for $T { }
  };
}

impl<T: FLocal> FLocal for *const T { }
impl<T: FLocal> FLocal for *mut T { }
impl<T: FLocal> FLocal for &T { }
impl<T: FLocal> FLocal for &mut T { }

macro_rules! impl_primitive {
  ($T:ty) => {
    impl_f_as_proxy!($T);
    impl_f_local!($T);
  };
}

impl_primitive!(());
impl_primitive!(usize);



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

impl FAsProxy<'_> for u128 {
  type FSelf = Self;
}
impl FAsProxy<'_> for U128 {
  type FSelf = u128;
}


#[cfg(test)]
pub mod proxy {

  use super::U128;

  #[test]
  fn test_u128() {
    let u = 123u128;
    println!("{u}");
    let u2 = U128::from(u);
    println!("l: {}, r: {}", u2.l, u2.r);
    assert!(u == u128::from(u2));
  }

}

