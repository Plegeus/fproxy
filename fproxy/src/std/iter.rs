
use std::marker::PhantomData;
use libloading::{Library, Symbol};
use crate::{FAsProxy, FProxyFrom, FToC};


type RustIterator<T> = Box<dyn Iterator<Item = T>>;

/// Associate a proxy to rust's iterators.
/// `T` is the rust type returned from the function implemented
/// on the type inside the library.
/// `T` will be converted to a `CType`, which then needs to be
/// converted to a `Proxy`.
impl<'l, T> FAsProxy<'l> for RustIterator<T> 
where 
  T: FToC + FAsProxy<'l>,
{
  type FSelf = FIterator<'l, T::FSelf, T::CType>;
}

impl<T: FToC> FToC for RustIterator<T> {
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    let cont = FIterContainer::new(self);
    let fdyn_iter = Box::new(cont) as Box<dyn FDynIterator>;
    let boxed = Box::new(fdyn_iter);
    Box::into_raw(boxed) as *const ()
  }
}


/// The proxy to an iterator.
pub struct FIterator<'l, Proxy, CType> {
  proxy_marker: PhantomData<Proxy>,
  c_marker: PhantomData<CType>,
  handle: *const (), // pointer to FIterContainer
  lib: &'l Library,
}
impl<'l, Proxy, CType> FProxyFrom<'l, *const ()> for FIterator<'l, Proxy, CType> {
  fn proxy_from(handle: *const (), lib: &'l Library) -> Self {
    FIterator { 
      proxy_marker: PhantomData, 
      c_marker: PhantomData, 
      handle, 
      lib, 
    }
  }
}


/// A wrapper around an iterator, store next to allow 
/// for a pointer to be passed over the dll boundary.
/// Always remains within the dll.
pub struct FIterContainer<T: FToC> {
  iterator: RustIterator<T>, // The iterator itself.
  next: Option<T::CType>, // The next item.
}
impl<T: FToC> FIterContainer<T> {
  fn new(iterator: RustIterator<T>) -> Self {
    FIterContainer { 
      iterator, 
      next: None,
    }
  }
}

impl<T: FToC> FDynIterator for FIterContainer<T> {
  fn next(&mut self) -> *const () {
    self.next = self.iterator.next().map(FToC::to_c);
    if let Some(item) = self.next.as_ref() {
      item as *const T::CType as *const ()
    } else {
      std::ptr::null()
    }
  }
}


/// Trait specifically designed for `FIterContainer`.
trait FDynIterator {
  fn next(&mut self) -> *const ();
}

impl<'l, Proxy, CType> Iterator for FIterator<'l, Proxy, CType> 
where 
  Proxy: FProxyFrom<'l, CType>
{
  type Item = Proxy;
  fn next(&mut self) -> Option<Self::Item> {
    unsafe {
      // To mimic enums, we can use pointers to 
      // indicate None values with null.
      // The function needs to return a pointer anyway 
      // (C's equivalent of Box<dyn Any>).
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FIterator_next\0").unwrap();
      let ptr = func(self.handle) as *const CType;
      if ptr.is_null() {
        return None;
      }
      // Safety: FIterator stores the result of next, but when
      // next is called again, the previous result is overridden.
      // The behavious below mimics a move.
      let c_value: CType = std::mem::transmute_copy(&*ptr);
      Some(FProxyFrom::proxy_from(c_value, &self.lib))
    }
  }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FIterator_next(handle: *const ()) -> *const () {
  let fiter = unsafe { &mut *(handle as *mut Box<dyn FDynIterator>) };
  fiter.next()
}







