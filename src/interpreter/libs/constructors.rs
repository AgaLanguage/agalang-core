use std::collections::HashMap;

use crate::compiler::{Object, Value};
use crate::functions_names;
use crate::util::{OnError, OnSome};

pub const LIB_NAME: &str = ":constructores";
const CADENA: &str = "cadena";

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    CADENA.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{CADENA}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, thread, _| {
          let value = args
            .first()
            .on_error(|_| format!("{CADENA}: Se esperaba un argumento"))?
            .clone();
          let value = match value {
            Value::String(s) => return Ok(Value::String(s)),
            v => v,
          };
          value
            .get_instance_property(functions_names::STRING, thread)
            .or_else(|| {
              value
                .as_strict_buffer(thread)
                .ok()
                .on_some(|buffer| Value::String(String::from_utf8_lossy(&buffer).to_string()))
            })
            .on_error(|_| format!("{CADENA}: se esperaba un valor bufeable"))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
