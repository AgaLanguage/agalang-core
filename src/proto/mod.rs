use crate::cache_fn;
use crate::value::Value;

mod string;

cache_fn!{
  pub fn proto(value_type: &'static str) -> Value
  {
    match value_type {
      "cadena" => string::string_proto(),
      _ => panic!("Unknown type: {value_type}"),
    }
  }
}