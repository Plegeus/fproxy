

use fproxy;
use fproxy::FInit;


#[fproxy::proxy("lib")]
pub struct MyPlugin {
  data: u128,
  run: Box<dyn Fn(u128) -> u128>,
}
#[fproxy::imp]
impl MyPlugin {
  pub fn run(&mut self) {
    let data = self.data;
    println!(">>> data is {data}");
    let data = (self.run)(data);
    println!(">>> updated data to {data}");
    self.data = data;
  }
}

const FACTOR: u128 = 2;

impl FInit for MyPlugin {
  type In = u128;
  fn init(data: Self::In) -> Self {
    println!(">>> initialising {}", std::any::type_name::<Self>());
    MyPlugin { 
      data: data, 
      run: Box::new(|i| i * FACTOR),
    }
  }
}
impl Default for MyPlugin {
  fn default() -> Self {
    println!(">>> initialising {}", std::any::type_name::<Self>());
    MyPlugin { 
      data: 1, 
      run: Box::new(|i| i * FACTOR),
    }
  }
}










