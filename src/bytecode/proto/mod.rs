use super::{value::{Value, STRING_TYPE}, DataCache};
mod string;

pub fn proto(value_type: String, mut cache: DataCache) -> Value {
  if cache.has(&value_type) {
    return cache.get(&value_type);
  }
  let value = match value_type.as_str() {
    STRING_TYPE => string::string_proto(),
    _ => panic!("Tipo desconocido: {value_type}"),
  };
  cache.set(value_type, value.clone());
  return value;
}
