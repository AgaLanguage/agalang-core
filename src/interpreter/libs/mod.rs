use crate::{compiler::Value, interpreter::cache::DataCache};

mod console;
mod math;
mod time;

pub fn libs(lib_name: String, mut cache: DataCache, resolver: impl FnOnce(&str) -> Value) -> Value {
  if cache.has(&lib_name) {
    return cache.get(&lib_name);
  }
  let value = match lib_name.as_str() {
    math::MATH_LIB => math::math_lib(),
    console::CONSOLE_LIB => console::console_lib(),
    time::TIME_LIB => time::time_lib(),
    path => resolver(path),
  };
  cache.set(lib_name, value.clone());
  return value;
}
