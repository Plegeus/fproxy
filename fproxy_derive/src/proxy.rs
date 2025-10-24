
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

  let fstruct = proxy_fstruct(&ident, &input);
  
  quote! {
    #fstruct
    fproxy::impl_f_from_c!(impl fproxy, #ident);
    fproxy::impl_f_to_c!(impl fproxy, #ident);
    #ast
  }
    .into()
}

fn fident(ident: &Ident) -> Ident {
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
      #[derive(Clone, Copy)]
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

    impl fproxy::FProxy for #tfident<'_> {
      unsafe fn free(&mut self) {
        Box::from_raw(self.handle as *mut #final_ident);
      }
    }

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

fn imp_trait(ident: &Ident, ast: ItemTrait) -> Quote {

  let name = trait_imp_ident(ident);
  // only handles functions, ignore e.g., associated types, bad.
  let impls = ast.items
    .iter()
    .filter_map(|item| match item {
      TraitItem::Fn(f) => Some(f),
      _ => None,
    })
    .fold(Quote::new(), |q, item| quote!(#q #item));

  quote! {

    struct #name(Box<dyn #ident>);
    impl #ident for #name {
      fproxy::delegate::delegate! {
        to self.0 {
          #impls
        }
      }
    }

    fproxy::impl_f_from_c!(impl fproxy, #name);
    fproxy::impl_f_to_c!(impl fproxy, #name);


  }
}

/// Generates the name for a concrete type implementing trait `ident`.
pub(crate) fn trait_imp_ident(ident: &Ident) -> Ident {
  Ident::new(&format!("FImp{ident}"), ident.span())
}





