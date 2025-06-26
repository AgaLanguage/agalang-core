use std::collections::HashMap;

use crate::compiler::{Object, Value};
use crate::util::{OnError, OnSome};

pub const CONSTRUCTORS_LIB: &str = ":constructores";
const CADENA: &str = "cadena";

pub fn constructors_lib() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{CONSTRUCTORS_LIB}>"));

  hashmap.set_instance_property(
    CADENA.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{CONSTRUCTORS_LIB}>::{CADENA}"),
        path: format!("<{CONSTRUCTORS_LIB}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, thread, _| {
          args
            .first()
            .on_some_option(|i| i.as_strict_buffer(thread).ok())
            .on_some(|buffer| Value::String(String::from_utf8_lossy(&buffer).to_string()))
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
