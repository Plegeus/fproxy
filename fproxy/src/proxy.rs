
use std::{ops::{Deref, DerefMut}};
use libloading::Library;
use crate::{FProxyFrom, FToC};


pub enum FAllocated<'a, T: ?Sized> {
  Box(Box<T>),
  Ref(&'a T),
  RefMut(&'a mut T),
}

impl<'a, T: ?Sized> Deref for FAllocated<'a, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    match self {
      FAllocated::Box(b) => &**b,
      FAllocated::Ref(r) => *r,
      FAllocated::RefMut(r) => *r,
    }
  }
}
impl<'a, T: ?Sized> DerefMut for FAllocated<'a, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    match self {
      FAllocated::Box(b) => &mut **b,
      FAllocated::Ref(_) => panic!(),
      FAllocated::RefMut(r) => *r,
    }
  }
}

impl<T> Clone for FAllocated<'_, T> {
  fn clone(&self) -> Self {
    match self {
      FAllocated::Ref(r) => FAllocated::Ref(*r),
      _ => panic!(),
    }
  }
}


/// Trait to control proxies from the application. <br/>
/// Everything that is a proxy implements this trait,
/// it is automatically implemented with `#[fproxy::proxy]`.
pub trait FProxy: FFree { }

/// For managing memory allocation on proxies and values
/// within proxies.
pub trait FFree {
  /// Frees allocated memory in the library. </br>
  /// ### Safety ###
  /// The caller must ensure `self` is dropped after.
  unsafe fn free(&mut self) {

  }
}

impl<T: FFree> FProxy for T { }


/// Owned representation of a proxy. <br/>
/// When `Self` is dropped, the proxy will be freed.
#[derive(Clone)]
pub struct FOwned<F: FProxy> {
  pub proxy: F,
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

unsafe impl<F> FToC for FOwned<F> 
where 
  F: FToC + FFree,
{
  type CType = F::CType;
  fn to_c(self) -> Self::CType {
    self.proxy.to_c()
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
  pub proxy: F,
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






