
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn};


pub(crate) fn fun(_: TokenStream, input: ItemFn) -> TokenStream {

  quote! {
    #input
  }
    .into()
}



