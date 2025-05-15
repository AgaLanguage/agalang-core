use super::{value::Value, DataCache};

mod math;

pub fn libs(lib_name: String, mut cache: DataCache) -> Value {
  if cache.has(&lib_name) {
    return cache.get(&lib_name);
  }
  let value = match lib_name.as_str() {
    math::MATH_LIB => math::math_lib(),
    _ => panic!("Libreria desconocida: {lib_name}"),
  };
  cache.set(lib_name, value.clone());
  return value;
}
