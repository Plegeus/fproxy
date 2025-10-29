

use my_plugin;
use my_plugin::FMyPlugin;

use fproxy::FLib;


fn main() {
    
  // Create a (proxy to a) plugin from a dynamicly loaded library.
  // Proxies are annoteted with an `F` for `foreign`.

  // The path to the library must be given, without file extension of `lib` prefix
  // in order to keep cross-platform compatibility.
  // Other parameters defined in a constructor follow the library.
  let lib = FLib::new("./target/debug/my_plugin");
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
  

}
