
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as Quote};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Block, Expr, ExprBlock, FnArg, Ident, ImplItem, Item, ItemImpl, LitByteStr, Local, Pat, Stmt, Type};


pub(crate) fn imp(_: TokenStream, input: ItemImpl) -> TokenStream {

  let ident = match input.self_ty.as_ref() {
    Type::Path(type_path) => type_path.path.get_ident().expect("expected struct"),
    _ => panic!("expected struct"),
  };
  let funs = input.items
    .iter()
    .filter_map(|item| match item {
      ImplItem::Fn(fun) => Some(fun),
      _ => None,
    })
    .fold(Quote::new(), |mut q, fun| {
  
      let name = fun.sig.ident.to_string();
      let _fn = Ident::new(&format!("_fproxy_{ident}_{name}"), Span::call_site());
      let _fn_bytes = LitByteStr::new(_fn.to_string().as_bytes(), Span::call_site());

      let _in = names_and_types(&fun.sig.inputs);
      //assert!(_in.len() < fun.sig.inputs.len(), "functions with no self parameter not yet supported");
      let _out = fun.sig.output.clone();

      let mut f = fun.clone();
      f.attrs.clear();
      f.block = syn::parse(
        quote! {
          {
            unsafe {
              let func: fproxy::libloading::Symbol<unsafe extern "C" fn() #_out> = 
                self.lib.get(#_fn_bytes).unwrap();
              func()
            }
          }
        }
          .into()
      )
        .expect("failed to parse Block");
      
      q.extend(quote! {
        impl ffi::FIdent {
          #f
        }
        // FIXME: find a way to guarantee/enforce unique names for foreign functions.
        #[unsafe(no_mangle)]
        unsafe extern "C" fn #_fn() {
          println!("WOW");
        }
      });
      q
    });
  

  quote! {
    #input
    #funs
  }
    .into()
}


fn remove_self(punct: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = &FnArg> {
  punct
    .iter()
    .filter(|arg| match arg {
      FnArg::Receiver(_) => false,
      _ => true,
    })
}
fn names_and_types(punct: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = (Ident, Type)> {
  remove_self(punct)
    .map(|arg| {
      let FnArg::Typed(typed) = arg else { panic!() }; // should be fine since self is removed
      let Pat::Ident(ident) = typed.pat.as_ref() else { panic!() }; // can function parameters be anything else?
      (ident.ident.clone(), typed.ty.as_ref().clone())
    })
}

