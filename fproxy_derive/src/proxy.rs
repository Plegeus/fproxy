
use proc_macro::{TokenStream};
use proc_macro2::{Literal, Span, TokenStream as Quote};
use quote::quote;
use syn::{DeriveInput, Ident, ItemTrait, LitByteStr, Token, TraitItem};
use syn::parse::{Parse, ParseStream};

use crate::tfident;


struct Input {
  lib: bool,
  is_trait: bool,
}
impl Input {
  fn as_trait() -> Self {
    let mut input = Input::default();
    input.is_trait = true;
    input
  }
}

impl Parse for Input {
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
    Ok(Input{
      lib: vec.contains(&format!("lib")),
      is_trait: false,
    })
  }
}
impl Default for Input {
  fn default() -> Self {
    Input { 
      lib: false,
      is_trait: false,
    }
  }
}


pub(crate) fn proxy(args: TokenStream, ast: DeriveInput) -> TokenStream {
  
  let ident = ast.ident.clone();
  let input: Input = syn::parse(args)
    .expect("failed to parse args");
  let tfident = tfident(&ident);

  let fstruct = proxy_fstruct(&ident, &input);
  
  let cfree: Ident = Ident::new(&format!("_fproxy_{ident}_FProxy_free"), ident.span());
  let cfree_bytes = LitByteStr::new(cfree.to_string().as_bytes(), Span::call_site());
  
  quote! {
    
    #fstruct
    
    fproxy::impl_f_from_c!(impl fproxy, #ident);
    fproxy::impl_f_to_c!(impl fproxy, #ident);

    impl fproxy::FProxy for #tfident<'_> {
      unsafe fn free(&mut self) {
        let func: fproxy::libloading::Symbol<unsafe extern "C" fn(*const ())> = 
          self.lib.get(#cfree_bytes).unwrap();
        func(self.handle);
      }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn #cfree(handle: *const ()) {
      Box::from_raw(handle as *mut #ident);
    }

    #ast
  }
    .into()
}
pub(crate) fn proxy_trait(args: TokenStream, ast: ItemTrait) -> TokenStream {
  
  let ident = &ast.ident;
  let fstruct = proxy_fstruct(ident, &Input::as_trait());
  let imp: Quote = crate::imp::imp_trait(args, ast.clone()).into();
  let imp_trait = imp_trait(ident, ast.clone());

  quote! {
    #fstruct 
    #imp_trait
    #imp
    #ast
  }
    .into()
}

pub(crate) fn fident(ident: &Ident) -> Ident {
  Ident::new(&format!("F{ident}"), ident.span())
}
fn proxy_fstruct(ident: &Ident, input: &Input) -> Quote {

  let fident = fident(ident);
  // e.g., type TFMyPlugin = FMyPlugin<'_>;
  // needed to access a proxy's type without worrying about lifetimes.
  let tfident = tfident (&ident);

  let _struct = if input.lib {
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
      pub struct #fident<'l> {
        pub handle: *const (),
        pub lib: &'l fproxy::libloading::Library,
      }
    )
  };


  let mut proxy_from = Quote::new();
  if !input.lib {
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

  let mut final_ident = quote!(#ident);
  if input.is_trait {
    final_ident = quote!(Box<dyn #final_ident>);
  }
  
  quote! {

    #_struct
    #proxy_from

    impl<'l> fproxy::FAsProxy<'l> for #final_ident {
      type FSelf = fproxy::FOwned<#tfident<'l>>;
    }
    impl<'l> fproxy::FAsProxy<'l> for &#final_ident {
      type FSelf = fproxy::FRef<#tfident<'l>>;
    }
    impl<'l> fproxy::FAsProxy<'l> for &mut #final_ident {
      type FSelf = fproxy::FRefMut<#tfident<'l>>;
    }

  }
}

fn imp_trait(ident: &Ident, _: ItemTrait) -> Quote {
  
  let tfident = tfident(ident);

  let cfree: Ident = Ident::new(&format!("_fproxy_{ident}_FProxy_free"), ident.span());
  let cfree_bytes = LitByteStr::new(cfree.to_string().as_bytes(), Span::call_site());
  
  quote! {

    impl fproxy::FProxy for #tfident<'_> {
      unsafe fn free(&mut self) {
        let func: fproxy::libloading::Symbol<unsafe extern "C" fn(*const ())> = 
          self.lib.get(#cfree_bytes).unwrap();
        func(self.handle);
      }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn #cfree(handle: *const ()) {
      use fproxy::FAllocated;
      let falloc = handle as *mut FAllocated<dyn #ident>;
      Box::from_raw(falloc);
    }
    

    impl<'l> fproxy::FAsProxy<'l> for &dyn #ident {
      type FSelf = fproxy::FRef<#tfident<'l>>;
    }
    impl<'l> fproxy::FAsProxy<'l> for &mut dyn #ident {
      type FSelf = fproxy::FRef<#tfident<'l>>;
    }


    impl fproxy::FToC for Box<dyn #ident> {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        Box::into_raw(
          Box::new(fproxy::FAllocated::<dyn #ident>::Box(self))
        ) as *const ()
      }
    }

    impl fproxy::FToC for &dyn #ident {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        Box::into_raw(
          Box::new(fproxy::FAllocated::<dyn #ident>::Arc(std::sync::Arc::new(self)))
        ) as *const ()
      }
    }
    impl fproxy::FToC for &mut dyn #ident {
      type CType = *const ();
      fn to_c(self) -> Self::CType {
        Box::into_raw(
          Box::new(fproxy::FAllocated::<dyn #ident>::ArcMut(std::sync::Arc::new(self)))
        ) as *const ()
      }
    }
    impl fproxy::FFromC for &dyn #ident {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        use fproxy::FAllocated;
        let falloc = &*(c_type as *const FAllocated::<dyn #ident>);
        match falloc {
          FAllocated::Box(b) => &**b,
          FAllocated::Arc(a) => **a,
          _ => panic!("proxy.rs, fn imp_trait in `impl fproxy::FFromC for &dyn #ident`"),
        }
      }
    }
    impl fproxy::FFromC for &mut dyn #ident {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        use fproxy::FAllocated;
        let falloc = &mut *(c_type as *mut FAllocated::<dyn #ident>);
        match falloc {
          FAllocated::Box(b) => &mut **b,
          FAllocated::ArcMut(a) => {
            use std::sync::Arc;
            use std::ops::Deref;
            *((*a).deref() as *const Self as *mut Self) // This is cursed.
          },
          _ => panic!("proxy.rs, fn imp_trait in `impl fproxy::FFromC for &mut dyn #ident`"),
        }
      }
    }

  }
}





