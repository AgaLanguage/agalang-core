mod string;

pub fn proto(
  value_type: String,
  mut cache: super::cache::DataCache,
) -> Option<crate::compiler::Value> {
  if cache.has(&value_type) {
    return Some(cache.get(&value_type));
  }
  let value = match value_type.as_str() {
    crate::compiler::STRING_TYPE => string::string_proto(),
    _ => return None,
  };
  cache.set(value_type, value.clone());
  return Some(value);
}
