

use my_plugin;
use my_plugin::FMyPlugin;


fn main() {
    
  // Create a (proxy to a) plugin from a dynamicly loaded library.
  // Proxies are annoteted with an `F` for `foreign`.

  // The path to the library must be given, without file extension of `lib` prefix
  // in order to keep cross-platform compatibility.
  // Other parameters defined in a constructor follow the library.
  let mut plug = unsafe { FMyPlugin::new("./target/debug/my_plugin", 5) };

  /*
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
 */
  let mut my_trait = plug.get_trait();
  my_trait.do_something();
  let mut my_trait = plug.get_trait_ref();
  //my_trait.do_something(); // cannot borrow as mutable!
  my_trait.do_something_else(123);
  

}
