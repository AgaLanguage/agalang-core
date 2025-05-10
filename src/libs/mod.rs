use crate::util::cache::DataManager;
use crate::value::Value;

mod math;

pub fn libs(lib_name: String, mut cache: DataManager<String, Value>) -> Value {
  if let Some(value) = cache.get(&lib_name) {
    return value.clone();
  } else {
    let value = match lib_name.as_str() {
      math::MATH_LIB => math::math_lib(),
      _ => panic!("Unknown lib: {lib_name}"),
    };
    cache.set(lib_name, value.clone());
    return value;
  }
}
