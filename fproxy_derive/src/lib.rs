
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


#[proc_macro_attribute]
pub fn repr_c(_: TokenStream, input: TokenStream) -> TokenStream {
  let input: DeriveInput = syn::parse(input).unwrap();
  let ident = &input.ident;
  quote! {
    
    #[fproxy::safer_ffi::derive_ReprC]
    #[repr(C)]
    #input

    impl fproxy::FLocal for #ident { }
    impl fproxy::FReprC for #ident { }

    impl fproxy::FAsProxy<'_> for #ident {
      type FSelf = Self;
    }

    unsafe impl fproxy::FToC for &#ident {
      type CType = *const #ident;
      fn to_c(self) -> Self::CType {
        self
      }
    }
    unsafe impl fproxy::FToC for &mut #ident {
      type CType = *mut #ident;
      fn to_c(self) -> Self::CType {
        self
      }
    }
    
    impl fproxy::FFromC for &#ident {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        unsafe { &*c_type }
      }
    }
    impl fproxy::FFromC for &mut #ident {
      unsafe fn from_c(c_type: Self::CType) -> Self {
        unsafe { &mut *c_type }
      }
    }

    impl fproxy::FFrom<&#ident> for #ident {
      fn ffrom(value: &#ident) -> Self {
        value.clone()
      }
    }

  }
    .into()
}








