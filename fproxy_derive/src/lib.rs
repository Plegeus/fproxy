
mod proxy;
mod imp;
mod fun;

use proc_macro::{TokenStream};
use quote::quote;
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

/*
#[proc_macro_derive(FIntoProxy)]
pub fn f_into_proxy(input: TokenStream) -> TokenStream {
  
  let input: DeriveInput = syn::parse(input)
    .expect("failed to parse input");
  let ident = input.ident;
  let tfident = tfident(&ident);

  quote! { 
    fproxy::impl_f_into_proxy!(impl fproxy, #ident, #tfident<'_>);
  }
    .into()
} */

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








