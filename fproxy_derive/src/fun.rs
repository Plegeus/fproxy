
use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItemFn};


pub(crate) fn fun(_: TokenStream, input: ImplItemFn) -> TokenStream {
  quote! {
    #input
  }
    .into()
}



