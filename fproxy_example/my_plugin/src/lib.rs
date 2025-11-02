#![allow(warnings)]

use fproxy;
use std::collections::HashMap;



#[fproxy::proxy]
#[derive(Default)]
pub struct Data {
  data: u128,
}

// When the proxy contains a "tag",
// only functions marked "tag" will be included.
#[fproxy::proxy("tag")]
impl Data {
  #[fproxy::proxy("tag")]
  pub fn read(&self) -> u128 {
    self.data
  }
} 


const FACTOR: u128 = 2;

// The `#[fproxy::proxy]` macro is the workhorse of the fproxy crate
// as it generates the proxies.
// Adding "lib" indicates the proxy is the owner of the Library.
// Ommiting "lib" still requires a library object, the proxy will store 
// a reference to a Library object.
#[fproxy::proxy("lib")]
pub struct MyPlugin {
  data: Data,
  map: HashMap<&'static str, u128>,
  counter: usize,
  run: Box<dyn Fn(u128) -> u128>,
}

const DATA: Data = Data { data: 8192 };

// To allow customisability, impl must be annotated with a similar macro call.
// By default, all functions in the `impl` are generated for the proxy, 
// unless there is no `self` parameter.
#[fproxy::proxy]
impl MyPlugin {

  pub fn assoc() -> &'static Data {
    &DATA
  }

  pub fn map(&self) -> &HashMap<&'static str, u128> {
    &self.map
  }

  // Tagging a function with "new" indicates this is a constructor for the proxy,
  // in which case the function will be implemented on the proxy.
  // The "lib" annotation indicates the proxy is the owner of the Library object,
  // the construtor for the proxy will include an additional argument for the library path.
  // If "lib" is ommited, it must also be ommited above, on the type definition and 
  // instead of a path argument, the constructor will take a reference to the library instead.
  #[fproxy::proxy("new", "lib")]
  pub fn new(data: u128) -> Self {
    //println!("creating plugin with data: {data}");
    MyPlugin { 
      data: Data { data }, 
      counter: 0,
      map: vec![
        ("one", 111),
        ("two", 222),
        ("three", 333),
        ("four", 444),
      ]
        .into_iter()
        .collect(),
      run: Box::new(|i| i * FACTOR),
    }
  }

  pub fn print(&self, s: &str) {
    println!("MyPlugin PRINTS: {s}");
  }

  // No tags, the function is included on the proxy.
  pub fn run(&mut self) {
    self.counter += 1;
    let data = self.data.data;
    println!(">>> data is {data}");
    let data = (self.run)(data);
    println!(">>> updated data to {data}");
    self.data.data = data;
  } 

  pub fn data(&self) -> &Data {
    &self.data
  }
  pub fn counter(&self) -> &usize {
    &self.counter
  }

  // Iterators are converted to an opaque type which in turn again 
  // Iterator.
  // The Item type will be set to the proxy type of the original item.
  pub fn iter(&self, n: u128) -> impl Iterator<Item = Data> {
    (0..n)
      .map(|i| Data { data: (self.run)(i) })
  } 


  // If for whichever reason, the proxy cannot or may not have access to a function,
  // the function can be ommited as whown below:
  #[fproxy::proxy("ignore")]
  pub fn something_complicated(&self) {
  
  }

  // Private functions are also ignored.
  fn something_private(&self) {
  
  } 


  pub fn get_trait(&self) -> impl MyTrait {
    Data::default()
  }
  pub fn get_trait_ref(&self) -> &dyn MyTrait {
    &self.data
  }
  pub fn get_trait_mut(&mut self) -> &mut dyn MyTrait {
    &mut self.data
  }

}



#[fproxy::proxy]
pub trait MyTrait {
  fn do_something(&mut self);
  fn do_something_else(&self, i: u128) {
    println!("doing something else with {i}");
  }
}
impl MyTrait for Data {
  fn do_something(&mut self) {
    self.data = 0;
    println!("Data is reset!");
  }
}

