use crate::{bytecode::ChunkGroup, value::Value};

pub fn string_proto() -> Value {
  println!("string_proto() called");
  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    "remplaza".into(),
    crate::value::Value::Object(crate::value::Object::Function(
      crate::value::Function::Native {
        name: "remplaza".into(),
        path: "proto/cadena".into(),
        func: |_, _| Value::Number(64.into()),
        chunk: ChunkGroup::default()
      },
    )),
  );
  Value::Object(crate::value::Object::Map(
    std::collections::HashMap::new().into(),
    hashmap.into(),
  ))
}
