use crate::{compiler::Value, interpreter::cache::DataCache};

mod console;
mod constructors;
mod fs;
mod math;
mod net;
mod time;

pub fn libs(lib_name: String, mut cache: DataCache, resolver: impl FnOnce(&str) -> Value) -> Value {
  if cache.has(&lib_name) {
    return cache.get(&lib_name);
  }
  let value = match lib_name.as_str() {
    constructors::LIB_NAME => constructors::lib_value(),
    console::LIB_NAME => console::lib_value(),
    math::LIB_NAME => math::lib_value(),
    time::LIB_NAME => time::lib_value(),
    net::LIB_NAME => net::lib_value(),
    fs::LIB_NAME => fs::lib_value(),
    path => resolver(path),
  };
  cache.set(lib_name, value.clone());
  value
}
