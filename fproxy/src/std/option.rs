
use libloading::Library;
use crate::{FAsProxy, FProxyFrom, FToC};


impl<'l, T> FAsProxy<'l> for Option<T> 
where 
  T: FAsProxy<'l>
{
  type FSelf = Option<T::FSelf>;
}


unsafe impl<T> FToC for Option<T> 
where 
  T: FToC
{
  type CType = *const ();
  fn to_c(self) -> Self::CType {
    let Some(t) = self else { return std::ptr::null(); };
    Box::into_raw(Box::new(t.to_c())) as *const ()
  }
}

impl<'l, T> FProxyFrom<'l, *const ()> for Option<T> 
where 
  T: FToC + FProxyFrom<'l, T::CType>
{
  fn proxy_from(ptr: *const (), lib: &'l Library) -> Self {
    if ptr.is_null() {
      return None;
    }
    let ptr = ptr as *mut T::CType;
    let val = unsafe { *Box::from_raw(ptr) };
    Some(T::proxy_from(val, lib))
  }
}










