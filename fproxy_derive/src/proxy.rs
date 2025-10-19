
use proc_macro::{TokenStream};
use proc_macro2::{Literal, Span, TokenStream as Quote};
use quote::quote;
use syn::{DeriveInput, Ident, LitByteStr, Token};
use syn::parse::{Parse, ParseStream};

use crate::tfident;



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


pub(crate) fn proxy(args: TokenStream, ast: DeriveInput) -> TokenStream {
  
  let ident = ast.ident.clone();
  let fident = Ident::new(&format!("F{ident}"), ident.span());
  let args: Args = syn::parse(args)
    .expect("failed to parse args");

  let fstruct = proxy_fstruct(&ident, &fident, &args);
  
  quote! {
    #fstruct
    #ast
  }
    .into()
}


fn proxy_fstruct(ident: &Ident, fident: &Ident, args: &Args) -> Quote {

  // e.g., type TFMyPlugin = FMyPlugin<'_>;
  // needed to access a proxy's type without worrying about lifetimes.
  let tfident = tfident (&ident);

  let _struct = if args.lib {
    quote!(
      type #tfident<'l> = #fident;
      pub struct #fident {
        pub handle: *const (),
        pub lib: fproxy::libloading::Library,
      }
    )
  } else {
    quote!(
      type #tfident<'l> = #fident<'l>;
      #[derive(Clone, Copy)]
      pub struct #fident<'l> {
        pub handle: *const (),
        pub lib: &'l fproxy::libloading::Library,
      }
    )
  };


  let mut proxy_from = Quote::new();
  if !args.lib {
    proxy_from = quote! {
      impl<'l> fproxy::FProxyFrom<'l, *const ()> for #tfident<'l> {
        fn proxy_from(handle: *const (), lib: &'l fproxy::libloading::Library) -> Self {
          Self {
            handle,
            lib,
          }
        }
      }
    };
  }

  quote! {
    
    #_struct

    impl fproxy::FProxy for #tfident<'_> {
      unsafe fn free(&mut self) {
        Box::from_raw(self.handle as *mut #ident);
      }
    }

    impl<'l> fproxy::FAsProxy<'l> for #ident {
      type FSelf = fproxy::FOwned<#tfident<'l>>;
    }
    impl<'l> fproxy::FAsProxy<'l> for &#ident {
      type FSelf = fproxy::FRef<#tfident<'l>>;
    }
    impl<'l> fproxy::FAsProxy<'l> for &mut #ident {
      type FSelf = fproxy::FRefMut<#tfident<'l>>;
    }

    #proxy_from

  }
}








