mod string;
mod function;

pub fn proto(
  value_type: String,
  mut cache: super::cache::DataCache,
) -> Option<crate::compiler::Value> {
  if cache.has(&value_type) {
    return Some(cache.get(&value_type));
  }
  let value = match value_type.as_str() {
    crate::compiler::FUNCTION_TYPE => function::prototype(),
    crate::compiler::STRING_TYPE => string::prototype(),
    _ => return None,
  };
  cache.set(value_type, value.clone());
  Some(value)
}
