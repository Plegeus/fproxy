
use std::{collections::HashMap, hash::Hash, marker::PhantomData};
use libloading::{Library, Symbol};
use safer_ffi::layout::CType;
use crate::{iter::FIterator, FAsProxy, FFree, FFromC, FProxyFrom, FRef, FToC};



impl<'l, K, V> FAsProxy<'l> for &'l HashMap<K, V> 
where 
  K: FToC + FAsProxy<'l>,
  V: FToC + FAsProxy<'l>,
{
  type FSelf = FRef<FHashMap<'l, K::FSelf, K::CType, V::FSelf, V::CType>>;
}

impl<'l, K: 'static, V: 'static> FToC for &HashMap<K, V> 
where 
  &'l K: FToC,
  &'l V: FToC,
  K: FFromC + Eq + Hash,
{
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    FHashMapContainer::from(self)
      .to_c()
  }
}
impl<'l, KProxy, KCtype, VProxy, VCtype> FProxyFrom<'l, *const ()> for FHashMap<'l, KProxy, KCtype, VProxy, VCtype> {
  fn proxy_from(handle: *const (), lib: &'l Library) -> Self {
    FHashMap { 
      _markers: (PhantomData, PhantomData, PhantomData, PhantomData), 
      handle, 
      lib, 
    }
  }
}


pub struct FHashMap<'l, KProxy, KCtype, VProxy, VCtype> {
  _markers: (PhantomData<KProxy>, PhantomData<KCtype>, PhantomData<VProxy>, PhantomData<VCtype>),
  handle: *const (),
  lib: &'l Library,
}
impl<'l, KProxy, KCtype, VProxy, VCtype> FHashMap<'l, KProxy, KCtype, VProxy, VCtype> 
where 
  (KCtype, VCtype): FToC,
{

  pub fn keys(&self) -> FKeys<'l, KProxy, KCtype> {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FHashMap_keys\0").unwrap();
      FKeys(FIterator::proxy_from(func(self.handle), self.lib))
    }
  }
  pub fn values(&self) -> FValues<'l, VProxy, VCtype> {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FHashMap_values\0").unwrap();
      FValues(FIterator::proxy_from(func(self.handle), self.lib))
    }
  }

  pub fn iter(&self) -> FIterator<'l, (KProxy, VProxy), <(KCtype, VCtype) as FToC>::CType> {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ()) -> *const ()> =
        self.lib.get(b"_fproxy_FHashMap_iter\0").unwrap();
      FIterator::proxy_from(func(self.handle), self.lib)
    }
  }

  //pub fn get(&self, key: &KProxy) -> Option<&VProxy> {
  //
  //}

  //pub fn values(&self) -> FValues<'l, V> {
  //  unimplemented!()
  //}
  //pub fn values_mut(&self) -> FValuesMut<'l, V> {
  //  unimplemented!()
  //}

}

impl<KProxy, KCtype, VProxy, VCtype> FFree for FHashMap<'_, KProxy, KCtype, VProxy, VCtype> {
  unsafe fn free(&mut self) {
    unsafe {
      let func: Symbol<unsafe extern "C" fn(*const ())> =
        self.lib.get(b"_fproxy_FHashMap_free\0").unwrap();
      func(self.handle);
    }
  }
}

impl<'l, KProxy, KCtype, VProxy, VCtype> From<FRef<FHashMap<'l, KProxy, KCtype, VProxy, VCtype>>> for HashMap<KProxy, VProxy> 
where 
  FIterator<'l, (KProxy, VProxy), <(KCtype, VCtype) as FToC>::CType>: Iterator<Item = (KProxy, VProxy)>,
  (KCtype, VCtype): FToC,
  KProxy: Eq + Hash
{
  fn from(fmap: FRef<FHashMap<'l, KProxy, KCtype, VProxy, VCtype>>) -> Self {
    fmap.iter()
      .collect()
  }
}




trait FDynHashMap<'l>: FFree {
  fn keys(&'l self) -> *const ();
  fn values(&'l self) -> *const ();
  fn iter(&'l self) -> *const ();
  //fn get(&'l self, ) -> *const ();
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
  fn keys(&'l self) -> *const () {
    let keys: Box<dyn Iterator<Item = &K>> = Box::new(self.map().keys());
    keys.to_c()    
  }
  fn values(&'l self) -> *const () {
    let keys: Box<dyn Iterator<Item = &V>> = Box::new(self.map().values());
    keys.to_c()    
  }
  fn iter(&'l self) -> *const () {
    let iter: Box<dyn Iterator<Item = (&K, &V)>> = Box::new(
      self.map()
        .keys()
        .map(|k| (k, &self.map()[k]))
    );
    iter.to_c()
  }
  // Can only have a ctype of *const (), 
  // returns pointer, null if None.
  // Create a trait FFromPtr which takes a pointer to 
  // a rust value in the dll, than looks up an extern "C"
  // conversion method to "cast" the pointer to its CType,
  // from which the proxy will be constructed.
  //fn get(&'l self) -> *const () {
  //  if let Some(val) = self.map().get() {
  //    std::ptr::null()
  //  } else {
  //    std::ptr::null()
  //  }
  //}
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



pub struct FKeys<'l, KProxy, KCType>(FIterator<'l, KProxy, KCType>);

impl<'l, KProxy, KCType> Iterator for FKeys<'l, KProxy, KCType> 
where 
  KProxy: FProxyFrom<'l, KCType>
{
  type Item = KProxy;
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }
}


pub struct FValues<'l, VProxy, VCType>(FIterator<'l, VProxy, VCType>);

impl<'l, VProxy, VCType> Iterator for FValues<'l, VProxy, VCType> 
where 
  VProxy: FProxyFrom<'l, VCType>
{
  type Item = VProxy;
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_keys(handle: *const ()) -> *const () {
  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
  map.keys()
}
#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_values(handle: *const ()) -> *const () {
  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
  map.values()
}
#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_iter(handle: *const ()) -> *const () {
  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
  map.iter()
}

//#[unsafe(no_mangle)]
//unsafe extern "C" fn _fproxy_FHashMap_get(handle: *const ()) -> *const () {
//  let map = unsafe { &*(handle as *const Box<dyn FDynHashMap>) };
//  map.get()
//}


#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_FHashMap_free(handle: *const ()) {
  unsafe {
    let map = &mut *(handle as *mut Box<dyn FDynHashMap>);
    map.free();
  }
}



/*

pub struct Iter<'l, K, V> {
  k_marker: PhantomData<K>,
  v_marker: PhantomData<V>,
  handle: *const (),
  lib: &'l Library,
}
impl<'l, K, V> FProxyFrom<'l, *const ()> for Iter<'l, K, V> {
  fn proxy_from(handle: *const (), lib: &'l Library) -> Self {
    Iter { 
      k_marker: PhantomData, 
      v_marker: PhantomData, 
      handle, 
      lib, 
    }
  }
}



impl<K, V> Iterator for Iter<'_, K, V> {
  type Item = (K, V);
  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}

impl<'l, K, V> IntoIterator for FHashMap<'l, K, V> {
  type Item = (K, V);
  type IntoIter = Iter<'l, K, V>;
  fn into_iter(self) -> Self::IntoIter {

    Iter::proxy_from(unimplemented!(), self.lib)
  }
}

impl<K, V> Into<HashMap<K, V>> for &FHashMap<'_, K, V> {
  fn into(self) -> HashMap<K, V> {
    unimplemented!()
  }
}

 */



