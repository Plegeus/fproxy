
use libloading::{Library, Symbol};

use crate::{convert::FFrom, iter::FIterator, FAsProxy, FFree, FFromC, FOwned, FProxyFrom, FRef, FToC};
use std::{collections::HashMap, hash::Hash, marker::PhantomData};


// Setup proxy.
impl<'l, K, V> FAsProxy<'l> for &'l HashMap<K, V> {
  type FSelf = FRef<FHashMap<'l, K, V>>;
}

impl<'l, K, V> From<FRef<FHashMap<'l, K, V>>> for HashMap<K::FSelf, V::FSelf> 
where 
  K: FAsProxy<'l> + 'l,
  V: FAsProxy<'l> + 'l,
  &'l K: FAsProxy<'l>,
  &'l V: FAsProxy<'l>,
  K::FSelf: FFrom<<&'l K as FAsProxy<'l>>::FSelf> + Hash + Eq,
  V::FSelf: FFrom<<&'l V as FAsProxy<'l>>::FSelf>,
  FIterator<'l, <(&'l K, &'l V) as FAsProxy<'l>>::FSelf>: Iterator<Item = <(&'l K, &'l V) as FAsProxy<'l>>::FSelf>
{
  fn from(fmap: FRef<FHashMap<'l, K, V>>) -> Self {
    fmap.iter()
      .map(|(k, v)| (FFrom::ffrom(k), FFrom::ffrom(v)))
      .collect()
  }
}


pub struct FHashMap<'l, K, V> {
  k_marker: PhantomData<K>,
  v_marker: PhantomData<V>,
  handle: *const (),
  lib: &'l Library,
}
impl<'l, K, V> FHashMap<'l, K, V> {
  pub fn iter(&self) -> FOwned<FIterator<'l, <(&'l K, &'l V) as FAsProxy<'l>>::FSelf>>
  where   
    &'l K: FAsProxy<'l>,
    &'l V: FAsProxy<'l>,
  {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FHashMap_iter\0").unwrap();
      FIterator::proxy_from(func(self.handle), self.lib)
        .into()
    }
  }
}

impl<K, V> FFree for FHashMap<'_, K, V> {
  unsafe fn free(&mut self) {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ())> =
        self.lib.get(b"_fproxy_FHashMap_free\0").unwrap();
      func(self.handle);
    }
  }
}
unsafe impl<'l, K, V> FToC for &'l HashMap<K, V> 
where
  K: Eq + Hash + 'static,
  V: 'static,
  &'l K: FToC,
  &'l V: FToC,
{
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    FHashMapContainer::from(self)
      .to_c()
  }
}
impl<'l, K, V> FProxyFrom<'l, *const ()> for FHashMap<'l, K, V> {
  fn proxy_from(handle: *const (), lib: &'l Library) -> Self {
    FHashMap { 
      k_marker: PhantomData,
      v_marker: PhantomData,
      handle, 
      lib, 
    }
  }
}

// Setup type containers.
trait FDynHashMap<'l>: FFree {
  fn iter(&'l self) -> *const ();
}

struct FHashMapContainer<K, V> {
  map: *const HashMap<K, V>,
}
impl<K, V> FHashMapContainer<K, V> {
  fn map(&self) -> &HashMap<K, V> {
    unsafe { &*self.map }
  }
}

impl<K, V> From<*const HashMap<K, V>> for FHashMapContainer<K, V> {
  fn from(map: *const HashMap<K, V>) -> Self {
    FHashMapContainer { 
      map,
    }
  }
}
impl<K, V> From<*mut HashMap<K, V>> for FHashMapContainer<K, V> {
  fn from(map: *mut HashMap<K, V>) -> Self {
    Self::from(map as *const HashMap<K, V>)
  }
}
impl<K, V> From<HashMap<K, V>> for FHashMapContainer<K, V> {
  fn from(map: HashMap<K, V>) -> Self {
    Self::from(Box::into_raw(Box::new(map)))
  }
}
impl<K, V> From<&HashMap<K, V>> for FHashMapContainer<K, V> {
  fn from(map: &HashMap<K, V>) -> Self {
    Self::from(map as *const HashMap<K, V>)
  }
}
impl<K, V> From<&mut HashMap<K, V>> for FHashMapContainer<K, V> {
  fn from(map: &mut HashMap<K, V>) -> Self {
    Self::from(map as *mut HashMap<K, V>)
  }
}


impl<'l, K, V> FDynHashMap<'l> for FHashMapContainer<K, V> 
where 
  K: Eq + Hash + 'static,
  V: 'static,
  &'l K: FToC,
  &'l V: FToC,
{
  fn iter(&'l self) -> *const () {
    let iter: Box<dyn Iterator<Item = (&K, &V)>> = Box::new(
      self.map()
        .keys()
        .map(|k| (k, &self.map()[k]))
    );
    iter.to_c()
  }
}

unsafe impl<'l, K, V> FToC for FHashMapContainer<K, V> 
where 
  K: Eq + Hash + 'static,
  V: 'static,
  &'l K: FToC,
  &'l V: FToC,
{
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    let b: Box<dyn FDynHashMap> = Box::new(self);
    Box::into_raw(Box::new(b)) as *const ()
  }
}
impl<K, V> FFree for FHashMapContainer<K, V> {
  unsafe fn free(&mut self) {
    unsafe {
      Box::from_raw(self.map as *mut HashMap<K, V>);
    }
  }
}






#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_free(handle: *const ()) {
  unsafe {
    let map = &mut *(handle as *mut Box<dyn FDynHashMap>);
    map.free();
  }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_iter(handle: *const ()) -> *const () {
  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
  map.iter()
}





