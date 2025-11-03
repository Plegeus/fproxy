
use libloading::{Library, Symbol};

use crate::{iter::FIterator, FAsProxy, FFree, FFromC, FRef, FToC};
use std::{collections::HashMap, hash::Hash, marker::PhantomData};


impl<'l, K, V> FAsProxy<'l> for &'l HashMap<K, V> {
  type FSelf = FRef<FHashMap<'l, K, V>>;
}

pub struct FHashMap<'l, K, V> {
  k_marker: PhantomData<K>,
  v_marker: PhantomData<V>,
  handle: *const (),
  lib: &'l Library,
}
impl<'l, K, V> FHashMap<'l, K, V> {

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


impl<'l, K: 'static, V: 'static> FDynHashMap<'l> for FHashMapContainer<K, V> 
where 
  &'l K: FToC,
  &'l V: FToC,
  K: FFromC + Eq + Hash,
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

impl<'l, K: 'static, V: 'static> FToC for FHashMapContainer<K, V> 
where 
  &'l K: FToC,
  &'l V: FToC,
  K: FFromC + Eq + Hash,
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
unsafe extern "C" fn _fproxy_FHashMap_iter(handle: *const ()) -> *const () {
  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
  map.iter()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_free(handle: *const ()) {
  unsafe {
    let map = &mut *(handle as *mut Box<dyn FDynHashMap>);
    map.free();
  }
}




