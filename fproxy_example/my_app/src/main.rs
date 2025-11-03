

use my_plugin;
use my_plugin::{FMyPlugin, FData};

use fproxy::{FLib, FRef, FOwned};
use fproxy::collections::{FHashMap};
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

  //let data = FMyPlugin::assoc(&lib);
  //println!("{}", data.read());

  let mut plug = unsafe { FMyPlugin::new(lib, 5) };

  // Iterators can also be turned into a proxy:
  //for data in plug.get_datas(4) {  
  //  let data: FOwned<FData<'_>> = data;
  //  println!("iter: {}", data.read());
  //}
  //for i in plug.get_i32s() {
  //  let i: i32 = i;
  //  println!("{i}");
  //}
  //for i in plug.get_refrs() {
  //  let i: &i32 = i;
  //  println!("{i}");
  //}


  /*
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


  let mut my_trait = plug.get_trait();
  my_trait.do_something();

  let my_trait = plug.get_trait_ref();
  {
    let my_trait2 = my_trait.clone();
    my_trait2.do_something_else(456);
  }

  my_trait.do_something_else(123);
 */

  //let map: FRef<FHashMap<'_, &str, _, i32, _>> = 
  //  plug.map();
  //let iter: FIterator<'_, &i32, _> = map.values();
  //for v in iter {
  //  println!("{v}");
  //}
  

  let map = plug.map();
  for (k, v) in map.iter() {
    println!("{k}: {v}");
  }

  let map = HashMap::from(map);
  println!("{map:?}");

}
