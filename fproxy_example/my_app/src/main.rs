

use my_plugin;
use my_plugin::ffi::FMyPlugin;


fn main() {
    
  let plug = unsafe { FMyPlugin::new("./target/debug/my_plugin.dll", 5).unwrap() };
  

}


