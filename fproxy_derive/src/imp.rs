
use std::{fmt::Display};

use proc_macro::{TokenStream};
use proc_macro2::{Literal, Span, TokenStream as Quote};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated, token::{Comma, Pub}, Attribute, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, ItemTrait, Lifetime, LitByteStr, Pat, Receiver, ReturnType, Signature, Token, TraitItem, TraitItemFn, Type, TypeImplTrait, TypeParamBound, Visibility};

use crate::tfident;


struct Args {
  tag: bool,
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
      tag: vec.contains(&format!("tag")),
    })
  }
}
impl Default for Args {
  fn default() -> Self {
    Args { tag: false }
  }
}

pub(crate) struct ItemDetails {
  args: Args,
  ident: Ident,
  is_trait: bool,
}
impl ItemDetails {
  fn from_impl(args: &TokenStream, input: &ItemImpl) -> Self {
    ItemDetails { 
      args: syn::parse(args.clone())
        .expect("failed to parse args"),
      ident: match input.self_ty.as_ref() {
        Type::Path(type_path) => type_path.path.get_ident().expect("expected struct").clone(),
        _ => crate::macro_panic!("expected struct"),
      }, 
      is_trait: false,
    }
  }
}

impl From<&ItemTrait> for ItemDetails {
  fn from(item: &ItemTrait) -> Self {
    ItemDetails { 
      args: Args::default(),
      ident: item.ident.clone(), 
      is_trait: true, 
    }
  }
}

impl Display for ItemDetails {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("{}", &self.ident))
  }
}
impl ToTokens for ItemDetails {
  fn to_tokens(&self, tokens: &mut Quote) {
    self.ident.to_tokens(tokens);
  }
}


pub(crate) struct FunctionDetails {
  // the elements of the function relevant to this macro.
  vis: Visibility,
  sig: Signature,
  // specific details about the function.
  tag: bool,
  slf: Option<Receiver>,  // The self parameter, if present.
  prv: bool,              // Whether the function is private.
  new: bool,              // Whether the function is constructor.
  lib: bool,              // Whether the library is owned by the proxy (needed for the constructor).
  ignore: bool,           // Whether the function is explicitly ignored.
  returns_impl: Option<Punctuated<TypeParamBound, Token![+]>>, // Whether the function returns `impl T`
}
impl FunctionDetails {

  fn new(attrs: Vec<Attribute>, vis: Visibility, sig: Signature) -> Self {
    
    // This is bad.
    let tag = attrs
      .iter()
      .filter(|atr| atr.to_token_stream().to_string().contains("fproxy"))
      .next()
      .map_or(String::new(), |atr| atr.to_token_stream().to_string());

    let mut fun = FunctionDetails {
        vis,
        sig,
        tag: tag.contains("tag"),
        slf: None,
        prv: false,
        new: tag.contains("new"),
        lib: tag.contains("lib"),
        ignore: tag.contains("ignore"),
        returns_impl: None,
    };
    
    if let Some(FnArg::Receiver(slf)) = fun.sig.inputs.first() {
      fun.slf = Some(slf.clone());
    }
    fun.prv = !matches!(fun.vis, Visibility::Public(_));

    if let ReturnType::Type(_, typ) = fun.sig.output.clone() {
      if let Type::ImplTrait(TypeImplTrait { impl_token: _, bounds }) = *typ {
        fun.returns_impl = Some(bounds);
      }
    }

    if fun.new && fun.slf.is_some() {
      crate::macro_panic!("function {} is annotated with #[fproxy::imp(\"new\"), it cannot have a self paramter", &fun.sig.ident);
    }
    if fun.prv && fun.tag {
      crate::macro_panic!("function {} is annotated with #[fproxy::imp(\"tag\"), but it is private", &fun.sig.ident);
    }

    fun
  }

  /// Given the ident of the type for which this function is 
  /// implemented, generate the `extern "C"` name.
  fn extern_c_name(&self, ident: &Ident) -> Ident {
    Ident::new(&format!("_fproxy_{ident}_{}", &self.sig.ident), ident.span())
  }
  fn output(&self, ident: &Ident, lt: Option<&str>) -> Quote {
    match self.sig.output.clone() {
      ReturnType::Default => quote!(()),
      ReturnType::Type(_, typ) => {
        let mut typ = *typ;
        typ = replace_lifetimes(&typ, lt.unwrap_or(""));
        //typ = replace_lifetimes(&typ, "'l");
        //if lifetime.is_none() {
        //  typ = replace_lifetimes(&typ, "");
        //}
        match &mut typ {
          Type::Path(path) if path.path.segments.first().unwrap().ident.to_string() == "Self" => {
            return quote!(#ident);
          },
          Type::Reference(refr) => {
            if let Some(lt) = lt {
              refr.lifetime = Some(Lifetime::new(lt, Span::call_site()));
            }
          }
          Type::ImplTrait(TypeImplTrait { impl_token: _, bounds }) => {
            return quote!(Box<dyn #bounds>);
          },
          _ => (),
        };
        return quote!(#typ);
      },
    }
  }

  fn should_ignore(&self, item: &ItemDetails) -> bool {
    //(!self.slf.is_some() && !self.new) ||
    self.prv || 
    self.ignore ||
    (item.args.tag && !self.tag)
  }
  fn self_as_trait(&self, item: &ItemDetails) -> Option<Quote> {
    if item.is_trait {
      if let Some(slf) = &self.slf {
        let refr = slf.reference.clone().map(|(and, _)| and);
        let mutb = &slf.mutability;
        return Some(quote! {#refr #mutb dyn #item});
      }
    } 
    None
  }


}

impl From<&ImplItemFn> for FunctionDetails {
  fn from(fun: &ImplItemFn) -> Self {
    FunctionDetails::new( 
      fun.attrs.clone(), 
      fun.vis.clone(), 
      fun.sig.clone(), 
    )
  }
}
impl From<&TraitItemFn> for FunctionDetails {
  fn from(fun: &TraitItemFn) -> Self {
    FunctionDetails::new(
      fun.attrs.clone(), 
      Visibility::Public(Pub::default()), 
      fun.sig.clone(), 
    )
  }
}


pub(crate) fn imp(args: TokenStream, input: ItemImpl) -> TokenStream {

  let item = ItemDetails::from_impl(&args, &input);
  let tfident = tfident(&item.ident);

  fn iter(input: &ItemImpl) -> impl Iterator<Item = FunctionDetails> {
    input.items
      .iter()
      .filter_map(|item| match item {
        ImplItem::Fn(fun) => Some(fun),
        _ => None,
      })
      .map(FunctionDetails::from)
  }

  let funs = make_funs(&item, iter(&input));
  let c_funs = make_c_funs(&item, iter(&input));

  quote! {

    impl #tfident<'_> {
      #funs
    }

    #input

    #c_funs

  }
    .into()
}
pub(crate) fn imp_trait(_: TokenStream, input: ItemTrait) -> TokenStream {

  let item = ItemDetails::from(&input);
  let tfident = tfident(&item.ident);

  fn iter(input: &ItemTrait) -> impl Iterator<Item = FunctionDetails> {
    input.items
      .iter()
      .filter_map(|item| {
        match item {
          TraitItem::Fn(fun) => {
            Some(FunctionDetails::from(fun))
          },
          _ => None,
        }
      })
  }

  let funs = make_funs(&item, iter(&input));
  let c_funs = make_c_funs(&item, iter(&input));


  quote! {

    impl #tfident<'_> {
      #funs
    }

    #c_funs

  }
    .into()
}

fn make_funs(ident: &ItemDetails, funs: impl Iterator<Item = FunctionDetails>) -> Quote {
  funs
    .map(|fun| imp_fun(ident, &fun))
    .fold(Quote::new(), |q, fun| quote!(#q #fun))
}
fn make_c_funs(ident: &ItemDetails, funs: impl Iterator<Item = FunctionDetails>) -> Quote {
  funs
    .map(|fun| imp_c_fun(ident, &fun))
    .fold(Quote::new(), |q, fun| quote!(#q #fun))
}

fn imp_fun(item: &ItemDetails, fun: &FunctionDetails) -> Quote {

  if fun.should_ignore(item) {
    return quote!();
  }

  let output = fun.output(&item.ident, None);
  let input = Input::from(fun);
  let mut input = input.fold(|q, (ident, typ)| {
    quote!(#q #ident: #typ)
  });

  // The function either needs a self (which has a library) or 
  // a library in order to execute in the dll.
  if let Some(mut slf) = fun.slf.clone() {
    if let Some((_, l)) = &mut slf.reference {
      *l = Some(Lifetime::new("'l", Span::call_site()));
    } 
    input = quote!(#slf, #input);
  } else {
    if fun.lib {
      input = quote!(lib: fproxy::FLib, #input);
    } else {
      input = quote!(lib: &'l fproxy::libloading::Library, #input);
    }
  }

  let name = &fun.sig.ident;
  let body = make_body(item, fun);

  // Inputs and outputs to proxies are proxies.
  quote! {
   pub fn #name<'l>(#input) -> <#output as fproxy::FAsProxy<'l>>::FSelf {
      #body
    }
  }
}
fn imp_c_fun(item: &ItemDetails, fun: &FunctionDetails) -> Quote {
  
  if fun.should_ignore(item) {
    return quote!();
  }

  let input = Input::from(fun);
  let fname = fun.extern_c_name(&item.ident);

  // The c function takes FReprC parameters, convert them back to 
  // the original rust types.
  //let mut input_names = input.names(|name| {
  //  quote!(fproxy::FFromC::from_c(#name))
  //});
  let mut input_names = input.patterns(|name, typ| {
    quote!(<#typ as fproxy::FFromC>::from_c(#name))
  });

  // The input needs to be FReprC.
  // The original input is mapped to there C types.
  let mut input = input.fold(|q, (ident, typ)| {
    quote!(#q #ident: <#typ as fproxy::FToC>::CType)
  });

  if fun.slf.is_some(){
    input_names = if let Some(slf) = fun.self_as_trait(item) {
      quote!(
        <#slf as fproxy::FFromC>::from_c(handle), 
        #input_names
      )
    } else {
      quote!(
        fproxy::FFromC::from_c(handle), 
        #input_names
      )
    };
    input = quote!(handle: *const (), #input);
  }

  let name = &fun.sig.ident;

  let body = if let Some(bounds) = &fun.returns_impl {
    let bounds: Type = syn::parse(quote!(Box<dyn #bounds>).into()).unwrap();
    let bounds = replace_lifetimes(&bounds, "");
    quote! {
      fproxy::FToC::to_c(
        Box::new(
          #item::#name(#input_names)
        ) as #bounds
      )
    }
  } else {
    quote! {
      fproxy::FToC::to_c(
        #item::#name(#input_names)
      )
    }
  };

  let output = fun.output(&item.ident, Some("'static"));

  quote! {
    /// Inputs and outputs to `extern "C"` functions are `CType`s.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn #fname(#input) -> <#output as fproxy::FToC>::CType {
      #body
    }
  } 
}

fn make_body(item: &ItemDetails, fun: &FunctionDetails) -> Quote {

  let input = Input::from(fun);
  let output = fun.output(&item.ident, Some("'static"));
  let fname = fun.extern_c_name(&item.ident);
  let fn_bytes = LitByteStr::new(fname.to_string().as_bytes(), Span::call_site());
  let input_names = input.names(|name| {
    quote!(fproxy::FToC::to_c(#name))
  });
  let input = input.patterns(|ident, typ| {
    quote!(#ident: <#typ as fproxy::FToC>::CType)
  });

  if fun.new {

    let mut _lib = quote! { lib };
    if fun.lib {
      _lib = quote!(lib.0);
    }

    quote! {
      {
        unsafe {
          use fproxy::libloading::{Library, Symbol};
          use fproxy::{FToC};
          let lib = #_lib;
          let func: Symbol<unsafe extern "C" fn(#input) -> <#item as FToC>::CType> = 
            lib.get(#fn_bytes).unwrap();
          Self {
            handle: func(#input_names),
            lib,
          }
            .into()
        }
      }
    }

  } else {

    let mut input = input;
    let mut func = quote!(func(#input_names));
    let mut lib = quote!(lib);
    if fun.slf.is_some() {
      input = quote!(*const (), #input);
      func = quote!(func(self.handle, #input_names));
      lib = quote!(self.lib);
    }

    quote! {
      {
        unsafe {
          use fproxy::libloading::{Symbol};
          let func: Symbol<unsafe extern "C" fn(#input) -> <#output as fproxy::FToC>::CType> = 
            #lib.get(#fn_bytes).unwrap();
          fproxy::FProxyFrom::<'l>::proxy_from(
            #func, 
            &#lib
          )
        }
      }
    }

  }
}


fn replace_lifetimes(ty: &Type, lt: &str) -> Type {
  let s = ty.to_token_stream().to_string();
  let re = Regex::new(r"'(?:[a-zA-Z0-9_]+|static)").unwrap();
  let s = re.replace_all(&s, lt).into_owned();
  syn::parse(s.parse().unwrap()).unwrap()
} 


/// Dismantles the input into tuples of `Ident` and `Type`. </br>
struct Input {
  names_and_types: Vec<(Ident, Type)>,
}
impl Input {

  //fn name(name: &Ident) -> Quote {
  //  quote!(#name)
  //}
  //fn pattern(ident: &Ident, typ: &Type) -> Quote {
  //  quote!(#ident: #typ)
  //}

  fn fold(&self, f: impl FnMut(Quote, &(Ident, Type)) -> Quote) -> Quote {
    self.names_and_types
      .iter()
      .fold(Quote::new(), f)
  }

  fn names(&self, f: impl Fn(&Ident) -> Quote) -> Quote {
    self.fold(|q, (ident, _)| {
      let name = f(ident);
      quote!(#q #name)
    })
  }
  fn patterns(&self, f: impl Fn(&Ident, &Type) -> Quote) -> Quote {
    self.fold(|q, (ident, typ)| {
      let pattern = f(ident, typ);
      quote!(#q #pattern)
    })
  }

}

impl From<&FunctionDetails> for Input { 
  fn from(fun: &FunctionDetails) -> Self {

    let punct = &fun.sig.inputs;

    /// Remove the `self` parameter and convert to identifiers and types. </br>
    /// The macro needs the type of `self` in order to do the requiered conversions. </br>
    fn remove_self(args: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = (Ident, Type)> {
      args
        .iter()
        .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
        .map(|arg| {
          let FnArg::Typed(typed) = arg else { crate::macro_panic!() }; // should be fine since self is removed
          let Pat::Ident(ident) = typed.pat.as_ref() else { crate::macro_panic!() }; // can function parameters be anything else?
          (ident.ident.clone(), typed.ty.as_ref().clone())
        })
    }

    Input { 
      names_and_types: remove_self(punct).collect()
    }
  }
}








