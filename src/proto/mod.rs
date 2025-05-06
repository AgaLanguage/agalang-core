use crate::util::cache::DataManager;
use crate::value::Value;

mod string;

pub fn proto(value_type: String, mut cache: DataManager<String, Value>) -> Value {
  if let Some(value) = cache.get(&value_type) {
    return value.clone();
  } else {
    let value = match value_type.as_str() {
      "cadena" => string::string_proto(),
      _ => panic!("Unknown type: {value_type}"),
    };
    cache.set(value_type, value.clone());
    return value;
  }
}
