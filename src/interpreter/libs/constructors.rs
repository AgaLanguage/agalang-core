use crate::compiler::{Object, Value};
use crate::functions_names;
use crate::util::OnError;

pub const LIB_NAME: &str = ":constructores";
const CADENA: &str = "Cadena";
const LIST: &str = "Lista";

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    CADENA,
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
            Value::Object(Object::Array(a)) => a
              .map_err(|v| v.as_strict_byte())
              .ok()
              .map(|buffer| Value::String(String::from_utf8_lossy(&buffer).to_string()))
              .on_error(|_| format!("{CADENA}: se esperaba un valor bufeable"))?,
            v => v,
          };
          value
            .get_instance_property(functions_names::STRING, thread)
            .or_else(|| Some(Value::String(value.as_string(thread))))
            .on_error(|_| format!("{CADENA}: se esperaba un valor bufeable"))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    LIST,
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{LIST}"),
        path: format!("<{LIB_NAME}>"),
        chunk: crate::compiler::ChunkGroup::default().into(),
        func: |_, args, _, _| Ok(Value::Object(Object::Array(args.into()))),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  Value::Object(Object::Map(Default::default(), hashmap.into()))
}
