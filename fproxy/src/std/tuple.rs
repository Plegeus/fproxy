
use crate::{convert::FReprC, FAsProxy, FFromC, FLocal, FProxyFrom, FToC};
use libloading::Library;
use safer_ffi::{derive_ReprC};


impl<'l, A, B> FAsProxy<'l> for (A, B,) 
where 
  A: FAsProxy<'l>,
  B: FAsProxy<'l>,
{
  type FSelf = (A::FSelf, B::FSelf);
}


#[derive_ReprC]
#[repr(C)]
pub struct FTuple {
  handle: *const (),
  len: usize,
}
impl FLocal for FTuple { }
impl FReprC for FTuple { }

impl<const N: usize> From<[*const (); N]> for FTuple {
  fn from(slice: [*const (); N]) -> Self {
    FTuple { 
      handle: Box::into_raw(Box::new(slice)) as *const (), 
      len: N, 
    }
  }
}
impl<const N: usize> From<FTuple> for [*const (); N] {
  fn from(value: FTuple) -> Self {
    unsafe { *Box::from_raw(value.handle as *mut Self) }
  }
}


impl<A> FToC for (A,) 
where 
  A: FToC
{
  type CType = FTuple;
  fn to_c(self) -> Self::CType {
    FTuple::from([Box::into_raw(Box::new(self.0.to_c())) as *const ()])
  }
}
impl<A, B> FToC for (A, B,) 
where 
  A: FToC,
  B: FToC,
{
  type CType = FTuple;
  fn to_c(self) -> Self::CType {
    FTuple::from([
      Box::into_raw(Box::new(self.0.to_c())) as *const (),
      Box::into_raw(Box::new(self.1.to_c())) as *const (),
    ])
  }
}



impl<A> FFromC for (A,) 
where 
  A: FToC + FReprC
{
  unsafe fn from_c(ftup: FTuple) -> Self {
    let ptrs: [*const (); 1] = From::from(ftup);
    unsafe {
      (*Box::from_raw(ptrs[0] as *mut A),)
    }
  }
}
impl<A, B> FFromC for (A, B,) 
where 
  A: FToC + FReprC,
  B: FToC + FReprC,
{
  unsafe fn from_c(ftup: FTuple) -> Self {
    let ptrs: [*const (); 2] = From::from(ftup);
    unsafe {
      (
        *Box::from_raw(ptrs[0] as *mut A),
        *Box::from_raw(ptrs[1] as *mut B),
      )
    }
  }
}


impl<'l, A> FProxyFrom<'l, FTuple> for (A,) 
where 
  A: FToC + FProxyFrom<'l, A::CType>,
{
  fn proxy_from(ftup: FTuple, lib: &'l Library) -> Self {
    let (a,): (A::CType,) = unsafe { FFromC::from_c(ftup) };
    (A::proxy_from(a, lib),)
  }
}
impl<'l, A, B> FProxyFrom<'l, FTuple> for (A, B,) 
where 
  A: FToC + FProxyFrom<'l, A::CType>,
  B: FToC + FProxyFrom<'l, B::CType>,
{
  fn proxy_from(ftup: FTuple, lib: &'l Library) -> Self {
    let (a, b,): (A::CType, B::CType,) = unsafe { FFromC::from_c(ftup) };
    (
      A::proxy_from(a, lib),
      B::proxy_from(b, lib),
    )
  }
}






