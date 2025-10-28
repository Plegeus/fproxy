
mod proxy;
mod imp;
mod fun;

use proc_macro::{TokenStream};
use syn::{DeriveInput, Ident, ImplItemFn, ItemImpl, ItemTrait};


macro_rules! macro_panic {
  () => {
    panic!("{}. {}", file!(), line!())
  };
  ($($arg:tt)*) => {
    {
      let msg = format!($($arg)*);
      panic!("{msg}, {}. {}", file!(), line!())
    }
  };
}
pub(crate) use macro_panic;


pub(crate) fn tfident(ident: &Ident) -> Ident {
  Ident::new(&format!("TF{ident}"), ident.span())
}


#[proc_macro_attribute]
pub fn proxy(args: TokenStream, input: TokenStream) -> TokenStream {
  if let Ok(input) = syn::parse::<DeriveInput>(input.clone()) {
    return proxy::proxy(args, input);
  }
  if let Ok(input) = syn::parse::<ItemTrait>(input.clone()) {
    return proxy::proxy_trait(args, input);
  }
  if let Ok(item) = syn::parse::<ItemImpl>(input.clone()) {
    return imp::imp(args, item);
  }
  if let Ok(item) = syn::parse::<ImplItemFn>(input.clone()) {
    return fun::fun(args, item);
  }
  panic!("expected impl of fn");
}








