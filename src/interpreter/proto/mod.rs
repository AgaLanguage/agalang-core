mod function;
mod string;

pub fn proto(
  value_type: &str,
  mut cache: super::cache::DataCache,
) -> Option<crate::compiler::Value> {
  let string = value_type.to_string();
  if cache.has(&string) {
    return Some(cache.get(&string));
  }
  let value = match value_type {
    crate::compiler::FUNCTION_TYPE | crate::compiler::NATIVE_FUNCTION_TYPE => function::prototype(),
    crate::compiler::STRING_TYPE => string::prototype(),
    _ => {
      return None;
    }
  };
  cache.set(string, value.clone());
  Some(value)
}
