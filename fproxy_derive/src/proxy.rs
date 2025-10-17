
use proc_macro::{TokenStream};
use proc_macro2::{Literal, Span, TokenStream as Quote};
use quote::quote;
use syn::{DeriveInput, Ident, LitByteStr, Token};
use syn::parse::{Parse, ParseStream};



struct Args {
  lib: bool,
}
impl Parse for Args {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut vec = Vec::new();
    loop {
      if let Ok(l) = input.parse::<Literal>() {
        vec.push(l.to_string().trim_matches('\"').to_string());
      }
      if input.parse::<Token![,]>().is_err() {
        break;
      }
    }
    Ok(Args{
      lib: vec.contains(&format!("lib")),
    })
  }
}


pub(crate) fn proxy(args: TokenStream, input: TokenStream) -> TokenStream {
  
  let ast: DeriveInput = syn::parse(input)
    .expect("failed to parse input");
  let ident = ast.ident.clone();
  let fident = Ident::new(&format!("F{ident}"), ident.span());
  let args: Args = syn::parse(args)
    .expect("failed to parse args");

  let _fstruct = proxy_fstruct(&ident, &fident, &args);
  
  quote! {
    pub mod ffi {
          
      use fproxy::{FProxy, FIntoProxy};
      use fproxy::libloading::{self, Library};

      #_fstruct

    }
    #ast
  }
    .into()
}


fn proxy_fstruct(ident: &Ident, fident: &Ident, args: &Args) -> Quote {

  let _struct = if args.lib {
    quote!(
      pub(in super) type FIdent = #fident;
      pub struct #fident {
        pub(in super) handle: *const (),
        pub(in super) lib: Library,
      }
    )
  } else {
    quote!(
      pub(in super) type FIdent = #fident<'f>;
      pub struct #fident<'f> {
        pub(in super) handle: *const (),
        pub(in super) lib: &'f Library,
      }
    )
  };


  quote! {
    
    #_struct

    impl FProxy for FIdent {
      unsafe fn free(&mut self) {
        Box::from_raw(self.handle as *mut super::#ident);
      }
    }
    impl FIntoProxy for FIdent {
      type FSelf = Self;
    }

  }
}





