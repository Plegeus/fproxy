
use libloading::Library;
use safer_ffi::derive_ReprC;
use crate::{FAsProxy, FLocal, FProxyFrom, FReprC, FToC};


impl<'l, Ok, Err> FAsProxy<'l> for Result<Ok, Err> 
where 
  Ok: FAsProxy<'l>,
  Err: FAsProxy<'l>,
{
  type FSelf = Result<Ok::FSelf, Err::FSelf>;
}


unsafe impl<Ok, Err> FToC for Result<Ok, Err> 
where 
  Ok: FToC,
  Err: FToC,
{
  type CType = FResult;
  fn to_c(self) -> Self::CType {
    match self {
      Ok(ok) => FResult::ok(ok),
      Err(err) => FResult::err(err),
    }
  }
}

#[derive_ReprC]
#[repr(C)]
pub struct FResult {
  ok: *const (),
  err: *const (),
}
impl FResult {
  fn ok<Ok>(ok: Ok) -> Self 
  where   
    Ok: FToC,
  {
    FResult { 
      ok: Box::into_raw(Box::new(ok.to_c())) as *const (), 
      err: std::ptr::null(),
    }
  }
  fn err<Err>(err: Err) -> Self 
  where 
    Err: FToC
  {
    FResult { 
      ok: std::ptr::null(),
      err: Box::into_raw(Box::new(err.to_c())) as *const (), 
    }
  }
}

impl FLocal for FResult { }
impl FReprC for FResult { }


impl<'l, Ok, Err> FProxyFrom<'l, FResult> for Result<Ok, Err> 
where 
  Ok: FToC + FProxyFrom<'l, Ok::CType>,
  Err: FToC + FProxyFrom<'l, Err::CType>,
{
  fn proxy_from(fresult: FResult, lib: &'l Library) -> Self {
    //if ptr.is_null() {
    //  return None;
    //}
    //let ptr = ptr as *mut T::CType;
    //let val = unsafe { *Box::from_raw(ptr) };
    //Some(T::proxy_from(val, lib))
    if !fresult.ok.is_null() {
      let ptr = fresult.ok as *mut Ok::CType;
      let val = unsafe { *Box::from_raw(ptr) };
      return Ok(Ok::proxy_from(val, lib));
    }
    if !fresult.err.is_null() {
      let ptr = fresult.err as *mut Err::CType;
      let val = unsafe { *Box::from_raw(ptr) };
      return Err(Err::proxy_from(val, lib));
    }
    panic!()
  }
}










