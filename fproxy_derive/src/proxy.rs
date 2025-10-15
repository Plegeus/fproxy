
use proc_macro::{TokenStream};
use proc_macro2::{Literal, Span, TokenStream as Quote};
use quote::quote;
use syn::{DeriveInput, Ident, LitByteStr, Token};
use syn::parse::{Parse, ParseStream};



struct Args {
  fprefix: String,
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
      fprefix: String::from("_fproxy"),
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
          
      use fproxy::{FInit, FProxy, FOwned};
      use fproxy::libloading::{self, Library, Symbol};

      type FIn = <super::#ident as FInit>::In;

      #_fstruct

    }
    #ast
  }
    .into()
}


fn proxy_fstruct(ident: &Ident, fident: &Ident, args: &Args) -> Quote {

  let span = Span::call_site();
  let _self = ident.to_string().to_ascii_lowercase();
  let _self = Ident::new(&_self, span);
  let _fn_new = Ident::new(&format!("{}_{ident}_new", &args.fprefix), span);
  let _fn_new_bytes = LitByteStr::new(_fn_new.to_string().as_bytes(), span);

  let _struct = if args.lib {
    quote!(
      pub(in super) type FIdent = #fident;
      pub struct #fident {
        handle: *const (),
        pub(crate) lib: Library,
      }
      impl #fident {
        pub unsafe fn new(lib: impl AsRef<std::ffi::OsStr>, input: FIn) -> Result<FOwned<Self>, libloading::Error> {
          let lib = Library::new(lib)?;
          let func: Symbol<unsafe extern "C" fn(FIn) -> *const ()> = 
            lib.get(#_fn_new_bytes).unwrap();
          Ok(
            Self {
              handle: func(input),
              lib,
            }
              .into()
          )
        }
      }
    )
  } else {
    quote!(
      pub(in super) type FIdent = #fident<'f>;
      pub struct #fident<'f> {
        handle: *const (),
        pub(crate) lib: &'f Library,
      }
      impl<'f> #fident<'f> {
        pub unsafe fn new<'l: 'f>(lib: &'l Library, input: FIn) -> Result<FOwned<Self>, libloading::Error> {
          let func: Symbol<unsafe extern "C" fn(FIn) -> *const ()> = 
            lib.get(#_fn_new_bytes).unwrap();
          Ok(
            Self {
              handle: func(input),
              lib,
            }
              .into()
          )
        }
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

    // FIXME: find a way to guarantee/enforce unique names for foreign functions.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn #_fn_new(input: FIn) -> *const () {
      let _self = super::#ident::init(input);
      Box::into_raw(Box::new(_self)) as *const ()
    }

  }
}





