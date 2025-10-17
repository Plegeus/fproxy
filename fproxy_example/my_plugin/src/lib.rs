

use fproxy;
use fproxy::{FToC, FIntoProxy, FFromC};


const FACTOR: u128 = 2;

// The `#[fproxy::proxy]` macro is the workhorse of the fproxy crate
// as it generates the proxies.
// Adding "lib" indicates the proxy is the owner of the Library.
// Ommiting "lib" still requires a library object, the proxy will store 
// a reference to a Library object.
#[fproxy::proxy("lib")]
#[derive(FIntoProxy, FToC, FFromC)]
pub struct MyPlugin {
  data: u128,
  run: Box<dyn Fn(u128) -> u128>,
}

// To allow customisability, impl must be annotated with a similar macro call.
// By default, all functions in the `impl` are generated for the proxy, 
// unless there is no `self` parameter.
#[fproxy::imp]
impl MyPlugin {

  // Tagging a function with "new" indicates this is a constructor for the proxy,
  // in which case the function will be implemented on the proxy.
  // The "lib" annotation indicates the proxy is the owner of the Library object,
  // the construtor for the proxy will include an additional argument for the library path.
  // If "lib" is ommited, it must also be ommited above, on the type definition and 
  // instead of a path argument, the constructor will take a reference to the library instead.
  #[fproxy::imp("new", "lib")]
  pub fn new(data: u128) -> Self {
    println!("creating plugin with data: {data}");
    MyPlugin { 
      data, 
      run: Box::new(|i| i * FACTOR),
    }
  }

  // No tags, the function is included on the proxy.
  pub fn run(&mut self) {
    let data = self.data;
    println!(">>> data is {data}");
    let data = (self.run)(data);
    println!(">>> updated data to {data}");
    self.data = data;
  }

  // If for whichever reason, the proxy cannot or may not have access to a function,
  // the function can be ommited as whown below:
  #[fproxy::imp("ignore")]
  pub fn something_complicated(&self) {

  }

  // Private functions are also ignored.
  fn something_private(&self) {

  }

}












