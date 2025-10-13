
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl};


pub(crate) fn imp(_: TokenStream, input: ItemImpl) -> TokenStream {

  quote! {
    #input
  }
    .into()
}

