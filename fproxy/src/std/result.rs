
use crate::FAsProxy;



impl<'l, T, E> FAsProxy<'l> for Result<T, E> 
where 
  T: FAsProxy<'l>,
  E: FAsProxy<'l>,
{
  type FSelf = Result<T::FSelf, E::FSelf>;
}


enum FResultVariant {
  Ok,
  Err,
}
struct FResult {
  val: *const (),
  
}







