
use std::{ops::{Deref, DerefMut}, sync::Arc};
use libloading::Library;
use crate::FProxyFrom;


pub enum FAllocated<'a, T: ?Sized> {
  Box(Box<T>),
  Arc(Arc<&'a T>),
  ArcMut(Arc<&'a mut T>),
}


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
  pub proxy: F,
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




