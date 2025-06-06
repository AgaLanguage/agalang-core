use super::{value::{Value, STRING_TYPE}, DataCache};
mod string;

pub fn proto(value_type: String, mut cache: DataCache) -> Option<Value> {
  if cache.has(&value_type) {
    return Some(cache.get(&value_type));
  }
  let value = match value_type.as_str() {
    STRING_TYPE => string::string_proto(),
    _ => return None,
  };
  cache.set(value_type, value.clone());
  return Some(value);
}
