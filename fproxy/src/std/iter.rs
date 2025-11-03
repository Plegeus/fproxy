
use std::{marker::PhantomData};
use libloading::{Library, Symbol};
use crate::{proxy::FFree, FAsProxy, FOwned, FProxy, FProxyFrom, FRef, FRefMut, FToC};


type RustIterator<'l, T> = Box<dyn Iterator<Item = T> + 'l>;

/// Associate a proxy to rust's iterators.
/// `T` is the rust type returned from the function implemented
/// on the type inside the library.
/// `T` will be converted to a `CType`, which then needs to be
/// converted to a `Proxy`.
impl<'l, T> FAsProxy<'l> for RustIterator<'_, T> {
  type FSelf = FOwned<FIterator<'l, T>>;
}
/// Analogous to `RustIterator<'l, T>`.
impl<'l, T> FAsProxy<'l> for &RustIterator<'_, T> {
  type FSelf = FRef<FIterator<'l, T>>;
}
/// Analogous to `RustIterator<'l, T>`.
impl<'l, T> FAsProxy<'l> for &mut RustIterator<'_, T> {
  type FSelf = FRefMut<FIterator<'l, T>>;
}

impl<I> Iterator for FOwned<I> 
where 
  I: Iterator + FProxy
{
  type Item = I::Item;
  fn next(&mut self) -> Option<Self::Item> {
    self.proxy.next()
  }
}
impl<I> Iterator for FRefMut<I> 
where 
  I: Iterator + FProxy
{
  type Item = I::Item;
  fn next(&mut self) -> Option<Self::Item> {
    self.proxy.next()
  }
}

impl<T: FToC> FToC for RustIterator<'_, T> {
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    FIterContainer::from(self)
      .to_c()
  }
}
impl<T: FToC> FToC for &RustIterator<'_, T> {
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    FIterContainer::from(self)
      .to_c()
  }
}
impl<T: FToC> FToC for &mut RustIterator<'_, T> {
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    FIterContainer::from(self)
      .to_c()
  }
}


/// The proxy to an iterator.
pub struct FIterator<'l, T> {
  marker: PhantomData<T>,
  handle: *const (), // pointer to FIterContainer
  lib: &'l Library,
}

impl<T> FFree for FIterator<'_, T> {
  unsafe fn free(&mut self) {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ())> =
        self.lib.get(b"_fproxy_FIterator_free\0").unwrap();
      func(self.handle);
    }
  }
}
impl<'l, T> FProxyFrom<'l, *const ()> for FIterator<'l, T> {
  fn proxy_from(handle: *const (), lib: &'l Library) -> Self {
    FIterator { 
      marker: PhantomData, 
      handle, 
      lib, 
    }
  }
}


/// A wrapper around an iterator, store next to allow 
/// for a pointer to be passed over the dll boundary.
/// Always remains within the dll.
pub struct FIterContainer<'l, T: FToC> {
  iter: *const RustIterator<'l, T>, // The iterator itself.
  next: Option<T::CType>, // The next item.
}

impl<T: FToC> FToC for FIterContainer<'_, T> {
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    let b: Box<dyn FDynIterator> = Box::new(self);
    Box::into_raw(Box::new(b)) as *const ()
  }
}

impl<'l, T: FToC> From<*mut RustIterator<'l, T>> for FIterContainer<'l, T> {
  fn from(iter: *mut RustIterator<'l, T>) -> Self {
    Self::from(iter as *const RustIterator<T>)
  }
}
impl<'l, T: FToC> From<*const RustIterator<'l, T>> for FIterContainer<'l, T> {
  fn from(iter: *const RustIterator<'l, T>) -> Self {
    FIterContainer { 
      iter, 
      next: None 
    }
  }
}
impl<'l, T: FToC> From<RustIterator<'l, T>> for FIterContainer<'l, T> {
  fn from(iter: RustIterator<'l, T>) -> Self {
    Self::from(Box::into_raw(Box::new(iter)))
  }
}
impl<'l, T: FToC> From<&RustIterator<'l, T>> for FIterContainer<'l, T> {
  fn from(iter: &RustIterator<'l, T>) -> Self {
    Self::from(iter as *const RustIterator<T>)
  }
}
impl<'l, T: FToC> From<&mut RustIterator<'l, T>> for FIterContainer<'l, T> {
  fn from(iter: &mut RustIterator<'l, T>) -> Self {
    Self::from(iter as *mut RustIterator<T>)
  }
}


impl<T: FToC> FFree for FIterContainer<'_, T> {
  unsafe fn free(&mut self) {
    unsafe {
      Box::from_raw(self.iter as *mut RustIterator<T>);
    }
  }
}
impl<T> FDynIterator for FIterContainer<'_, T> 
where 
  T: FToC,
{
  /// Returns `*const T::CType`. <br/>
  /// If the pointer is `null`, it **must** be interpreted
  /// as `None`.
  fn next(&mut self) -> *const () {
    let iter = unsafe { &mut *(self.iter as *mut RustIterator<T>) };
    self.next = iter.next().map(FToC::to_c);
    if let Some(item) = self.next.as_ref() {
      item as *const T::CType as *const ()
    } else {
      std::ptr::null()
    }
  }
}


/// Trait specifically designed for `FIterContainer`.
pub(in super) trait FDynIterator: FFree {
  fn next(&mut self) -> *const ();
}

impl<'l, T> Iterator for FIterator<'l, T> 
where 
  T: FAsProxy<'l> + FToC,
  T::FSelf: FProxyFrom<'l, T::CType>,
{
  type Item = T::FSelf;
  fn next(&mut self) -> Option<Self::Item> {
    unsafe {
      // To mimic enums, we can use pointers to 
      // indicate None values with null.
      // The function needs to return a pointer anyway 
      // (C's equivalent of Box<dyn Any>).
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FIterator_next\0").unwrap();
      let ptr = func(self.handle) as *const T::CType;
      if ptr.is_null() {
        return None;
      }
      // Safety: FIterator stores the result of next, but when
      // next is called again, the previous result is overridden.
      // The behavious below mimics a move.
      let c_value: T::CType = std::mem::transmute_copy(&*ptr);
      Some(FProxyFrom::proxy_from(c_value, &self.lib))
    }
  }
}



#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FIterator_next(handle: *const ()) -> *const () {
  let fiter = unsafe { &mut *(handle as *mut Box<dyn FDynIterator>) };
  fiter.next()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FIterator_free(handle: *const ()) {
  unsafe {
    let fiter = &mut *(handle as *mut Box<dyn FDynIterator>);
    fiter.free();
  }
}










