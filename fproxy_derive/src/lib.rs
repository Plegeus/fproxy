
mod proxy;
mod imp;
mod fun;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ImplItemFn, ItemFn, ItemImpl};


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



#[proc_macro_derive(FIntoProxy)]
pub fn f_into_proxy(input: TokenStream) -> TokenStream {
  
  let input: DeriveInput = syn::parse(input)
    .expect("failed to parse input");
  let ident = input.ident;

  quote! { 
    fproxy::impl_f_into_proxy!(impl fproxy, #ident, ffi::FIdent);
  }
    .into()
}

#[proc_macro_derive(FToC)]
pub fn f_to_c(input: TokenStream) -> TokenStream {
  
  let input: DeriveInput = syn::parse(input)
    .expect("failed to parse input");
  let ident = input.ident;

  quote! { 
    fproxy::impl_f_to_c!(impl fproxy, #ident);
  }
    .into()
}
#[proc_macro_derive(FFromC)]
pub fn f_from_c(input: TokenStream) -> TokenStream {
  
  let input: DeriveInput = syn::parse(input)
    .expect("failed to parse input");
  let ident = input.ident;

  quote! { 
    fproxy::impl_f_from_c!(impl fproxy, #ident);
  }
    .into()
}


#[proc_macro_attribute]
pub fn proxy(args: TokenStream, input: TokenStream) -> TokenStream {
  proxy::proxy(args, input)
}

#[proc_macro_attribute]
pub fn imp(args: TokenStream, input: TokenStream) -> TokenStream {
  if let Ok(item) = syn::parse::<ItemImpl>(input.clone()) {
    return imp::imp(args, item);
  }
  if let Ok(item) = syn::parse::<ImplItemFn>(input.clone()) {
    return fun::fun(args, item);
  }
  panic!("expected impl of fn");
}







