use crate::{compiler::Value, interpreter::cache::DataCache};

mod console;
mod constructors;
mod math;
mod net;
mod time;

pub fn libs(lib_name: String, mut cache: DataCache, resolver: impl FnOnce(&str) -> Value) -> Value {
  if cache.has(&lib_name) {
    return cache.get(&lib_name);
  }
  let value = match lib_name.as_str() {
    constructors::CONSTRUCTORS_LIB => constructors::constructors_lib(),
    console::CONSOLE_LIB => console::console_lib(),
    math::MATH_LIB => math::math_lib(),
    time::TIME_LIB => time::time_lib(),
    net::NET_LIB => net::net_lib(),
    path => resolver(path),
  };
  cache.set(lib_name, value.clone());
  return value;
}
