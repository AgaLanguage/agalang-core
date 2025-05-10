use crate::{bytecode::ChunkGroup, value::Value};

const REPLACE: &str = "remplaza";
const SPLIT: &str = "separa";
const BYTES: &str = "bytes";

pub fn string_proto() -> Value {
  let mut hashmap = std::collections::HashMap::new();

  hashmap.insert(
    REPLACE.into(),
    crate::value::Value::Object(crate::value::Object::Function(
      crate::value::Function::Native {
        name: REPLACE.into(),
        path: format!("<cadena>::{REPLACE}"),
        func: |this, args| {
          let old = args.get(0);
          if old.is_none() {
            return Err("remplaza: se esperaban 2 argumentos y se recibieron 0".into());
          }
          let old = old.unwrap().as_string();
          let new = args.get(1);
          if new.is_none() {
            return Err("remplaza: se esperaban 2 argumentos y se recibieron 1".into());
          }
          let new = new.unwrap().as_string();
          let string = this.as_string();
          let string = string.replace(&old, &new);
          Ok(Value::String(string.into()))
        },
        chunk: ChunkGroup::default(),
      },
    )),
  );

  hashmap.insert(
    BYTES.into(),
    crate::value::Value::Object(crate::value::Object::Function(
      crate::value::Function::Native {
        name: BYTES.into(),
        path: format!("<cadena>::{BYTES}"),
        func: |this, _| {
          let string = this.as_string();
          let list = string.as_bytes().iter().map(|b|Value::Byte(*b)).collect::<Vec<_>>();
          Ok(Value::Object(list.into()))
        },
        chunk: ChunkGroup::default(),
      },
    )),
  );

  hashmap.insert(
    SPLIT.into(),
    crate::value::Value::Object(crate::value::Object::Function(
      crate::value::Function::Native {
        name: SPLIT.into(),
        path: format!("<cadena>::{SPLIT}"),
        func: |this, args| {
          let separator = args.get(0);
          if separator.is_none() {
            return Err("remplaza: se esperaba 1 argumento y se recibieron 0".into());
          }
          let separator = separator.unwrap().as_string();
          let string = this.as_string();
          let list = string
            .split(&separator)
            .map(|s| Value::String(s.to_string()))
            .collect::<Vec<_>>();
          Ok(Value::Object(list.into()))
        },
        chunk: ChunkGroup::default(),
      },
    )),
  );
  Value::Object(crate::value::Object::Map(
    std::collections::HashMap::new().into(),
    hashmap.into(),
  ))
}
