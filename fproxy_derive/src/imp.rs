
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as Quote};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Lifetime, LitByteStr, Pat, Receiver, ReturnType, Type, Visibility};


pub(crate) fn imp(_: TokenStream, input: ItemImpl) -> TokenStream {

  let ident = match input.self_ty.as_ref() {
    Type::Path(type_path) => type_path.path.get_ident().expect("expected struct"),
    _ => crate::macro_panic!("expected struct"),
  };
  let funs = input.items
    .iter()
    .filter_map(|item| match item {
      ImplItem::Fn(fun) => Some(fun),
      _ => None,
    })
    .fold(Quote::new(), |mut q, fun| {
  
      let imp = Imp::from(fun);
      // No self parameter, and no constructor, ignore.
      if !imp.slf.is_some() && !imp.new {
        return q;
      }
      // The users wishes ignore the function or
      // the function is private.
      if imp.ignore || imp.prv {
        return q;
      }
      if imp.new && imp.slf.is_some() {
        crate::macro_panic!("functions annotated with #[fproxy::imp(\"new\") cannot have a self paramter");
      }

      let input = Input::from(&fun.sig.inputs);
      let output = make_output(ident, &fun.sig.output);

      let fname = &fun.sig.ident;
      let fname = Ident::new(&format!("_fproxy_{ident}_{fname}"), Span::call_site());

      let body = make_body(
        &imp, 
        ident,
        &fname,
        &input,
        &output,
      );
      let c_fun = make_c_function(
        &imp,
        ident,
        &fun.sig.ident,
        &fname,
        &input,
        &output,
      );

      let ffun = make_function(
        &imp, 
        &fun.sig.ident, 
        &input,
        &output, 
        &body
      );

      let tfident = Ident::new(&format!("TF{ident}"), ident.span());
      q.extend(quote! {
        impl #tfident<'_> {
          #ffun
        }
        // FIXME: find a way to guarantee/enforce unique names for foreign functions.
        #[unsafe(no_mangle)]
        #c_fun
      });

      q
    });
  

  quote! {
    #input
    #funs
  }
    .into()
}


fn make_function(imp: &Imp, name: &Ident, input: &Input, output: &Quote, body: &Quote) -> Quote {

  let mut input = input.fold(|q, (ident, typ)| {
    quote!(#q #ident: #typ)
  });

  // The function either needs a self (which has a library) or 
  // a library in order to execute in the dll.
  if let Some(mut slf) = imp.slf.clone() {
    if let Some((_, l)) = &mut slf.reference {
      *l = Some(Lifetime::new("'l", Span::call_site()));
    } 
    input = quote!(#slf, #input);
  } else {
    if imp.lib {
      input = quote!(lib: &str, #input);
    } else {
      input = quote!(lib: &'l fproxy::libloading::Library, #input);
    }
  }

  // Inputs and outputs to proxies are proxies.
  quote! {
   pub fn #name<'l>(#input) -> <#output as fproxy::FAsProxy<'l>>::FSelf {
      #body
    }
  }
}

fn make_c_function(imp: &Imp, ident: &Ident, fun: &Ident, fname: &Ident, input: &Input, output: &Quote) -> Quote {
  
  let mut input_names = input.names(|name| {
    quote!(fproxy::FFromC::from_c(#name))
  });
  let mut input = input.fold(|q, (ident, typ)| {
    quote!(#q #ident: <#typ as fproxy::FToC>::CType)
  });

  if !imp.new {
    input_names = quote!(fproxy::FFromC::from_c(handle), #input_names);
    input = quote!(handle: *const (), #input);
  }

  quote! {
    /// Inputs and outputs to `extern "C"` functions are `CType`s.
    unsafe extern "C" fn #fname(#input) -> <#output as fproxy::FToC>::CType {
      fproxy::FToC::to_c(#ident::#fun(#input_names))
    }
  } 
}
fn make_body(imp: &Imp, ident: &Ident, fname: &Ident, input: &Input, output: &Quote) -> Quote {

  let fn_bytes = LitByteStr::new(fname.to_string().as_bytes(), Span::call_site());
  let input_names = input.names(|name| {
    quote!(fproxy::FToC::to_c(#name))
  });
  let input = input.patterns(|ident, typ| {
    quote!(#ident: <#typ as fproxy::FToC>::CType)
  });

  if imp.new {

    let mut _lib = quote! { lib };
    if imp.lib {
      _lib = quote! {
        {
          let mut lib = std::path::PathBuf::from(lib);
          #[cfg(target_os = "windows")]
          lib.set_extension("dll");
          #[cfg(target_os = "macos")] {
            let name = lib.file_name().unwrap().to_str().unwrap();
            let name = format!("lib{name}");
            lib.set_file_name(name);
            lib.set_extension("dylib");
          }
          Library::new(lib.into_os_string().as_os_str()).unwrap()
        }
      };
    }

    quote! {
      {
        unsafe {
          use fproxy::libloading::{Library, Symbol};
          use fproxy::{FToC};
          let lib = #_lib;
          let func: Symbol<unsafe extern "C" fn(#input) -> <#ident as FToC>::CType> = 
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

    quote! {
      {
        unsafe {
          use fproxy::libloading::{Symbol};
          let func: Symbol<unsafe extern "C" fn(*const (), #input) -> <#output as fproxy::FToC>::CType> = 
            self.lib.get(#fn_bytes).unwrap();
          fproxy::FProxyFrom::<'l>::proxy_from(
            func(self.handle), 
            &self.lib
          )
        }
      }
    }

  }
}

fn make_output(ident: &Ident, out: &ReturnType) -> Quote {
  match out.clone() {
    ReturnType::Default => quote!(()),
    ReturnType::Type(_, mut typ) => {
      match typ.as_mut() {
        Type::Path(path) if path.path.segments.first().unwrap().ident.to_string() == "Self" => {
          return quote!(#ident);
        },
        Type::Reference(refr) => {
          refr.lifetime = Some(Lifetime::new("'static", Span::call_site()));
        }
        _ => (),
      };
      return quote!(#typ);
    },
  }
}


struct Imp {
  slf: Option<Receiver>,  // The self parameter, if present.
  prv: bool,              // Whether the function is private.
  new: bool,              // Whether the function is constructor.
  lib: bool,              // Whether the library is owned by the proxy (needed for the constructor).
  ignore: bool,           // Whether the function is explicitly ignored.
}
impl From<&ImplItemFn> for Imp {
  fn from(fun: &ImplItemFn) -> Self {
    // This is bad.
    let tag = fun.attrs
      .iter()
      .filter(|atr| atr.to_token_stream().to_string().contains("fproxy"))
      .next()
      .map_or(String::new(), |atr| atr.to_token_stream().to_string());
    Imp { 
      slf: if let Some(FnArg::Receiver(slf)) = fun.sig.inputs.first() {
        Some(slf.clone())
      } else {
        None
      },
      prv: !matches!(fun.vis, Visibility::Public(_)),
      new: tag.contains("new"), 
      lib: tag.contains("lib"), 
      ignore: tag.contains("ignore"),
    }
  }
}

/// Dismantles the input into tuples of `Ident` and `Type`. </br>
struct Input {
  names_and_types: Vec<(Ident, Type)>,
}
impl Input {

  fn name(name: &Ident) -> Quote {
    quote!(#name)
  }
  fn pattern(ident: &Ident, typ: &Type) -> Quote {
    quote!(#ident: #typ)
  }

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

impl From<&Punctuated<FnArg, Comma>> for Input { 
  fn from(punct: &Punctuated<FnArg, Comma>) -> Self {

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




