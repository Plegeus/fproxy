

use my_plugin;
use my_plugin::FMyPlugin;

use fproxy::{FLib, FRef};
use fproxy::collections::{FHashMap, FKeys};
use fproxy::iter::FIterator;
use fproxy::convert::{U128, FStr};
use std::collections::HashMap;


fn main() {
    
  // Create a (proxy to a) plugin from a dynamicly loaded library.
  // Proxies are annoteted with an `F` for `foreign`.

  // The path to the library must be given, without file extension of `lib` prefix
  // in order to keep cross-platform compatibility.
  // Other parameters defined in a constructor follow the library.
  #[cfg(target_os = "macos")]
  let lib = FLib::new("./target/debug/libmy_plugin.dylib");
  #[cfg(target_os = "windows")]
  let lib = FLib::new("./target/debug/my_plugin.dll");

  let data = FMyPlugin::assoc(&lib);
  println!("{}", data.read());

  let mut plug = unsafe { FMyPlugin::new(lib, 5) };

  plug.print("hello world!");

  // Functions defined on a foreign type also generate on the proxy.
  // Like this the dependant binary can use the foreign type with 
  // an almost one to one mapping.
  plug.run();
  plug.run();
  plug.run();
  plug.run();

  // Data is also defined as a proxy, 
  // the functions generated on Data are also available.
  let data = plug.data();
  println!("{}", data.read());
  println!("{}", plug.counter());

  // Iterators can also be turned into a proxy:
  for data in plug.iter(4) {  
    println!("iter: {}", data.read());
  }

  let mut my_trait = plug.get_trait();
  my_trait.do_something();

  let my_trait = plug.get_trait_ref();
  {
    let my_trait2 = my_trait.clone();
    my_trait2.do_something_else(456);
  }

  my_trait.do_something_else(123);

  let map = plug.map();
  let iter: FIterator<(&str, u128), _> = map.iter();
  for (k, v) in iter {
    println!("{k}: {v}");
  }
  let map = HashMap::from(map);
  println!("{map:?}");

}
