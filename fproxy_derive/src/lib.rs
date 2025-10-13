
mod proxy;
mod imp;
mod fun;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, ItemImpl};


#[proc_macro_derive(FInit)]
pub fn finit(input: TokenStream) -> TokenStream {
  
  let input: DeriveInput = syn::parse(input)
    .expect("failed to parse in put");
  let ident = input.ident;

  quote! { 
    fproxy::imp_finit!(#ident);
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
  if let Ok(item) = syn::parse::<ItemFn>(input.clone()) {
    return fun::fun(args, item);
  }
  panic!("expected impl of fn");
}




