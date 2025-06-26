use std::collections::HashMap;

use crate::compiler::{ChunkGroup, Function, Object, Value};

const REPLACE: &str = "remplaza";
const REPEAT: &str = "repite";
const SPLIT: &str = "separa";
const BYTES: &str = "bytes";

pub fn string_proto() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<cadena>"));

  hashmap.set_instance_property(
    REPLACE.into(),
    Value::Object(
      Function::Native {
        path: "".into(),
        name: format!("<cadena>::{REPLACE}"),
        func: |this, args, _, _| {
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
        chunk: ChunkGroup::default().into(),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    REPEAT.into(),
    Value::Object(
      Function::Native {
        path: "".into(),
        name: format!("<cadena>::{REPEAT}"),
        func: |this, args, _, _| {
          let count = match args.get(0) {
            None => Err("remplaza: se esperaban 2 argumentos y se recibieron 0".into()),
            Some(count) => count.as_number()?.floor().into(),
          }?;
          if count == 0 {
            return Ok(Value::String("".into()));
          }
          let string = this.as_string();
          let string = string.repeat(count);
          Ok(Value::String(string))
        },
        chunk: ChunkGroup::default().into(),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );

  hashmap.set_instance_property(
    BYTES.into(),
    Value::Object(
      Function::Native {
        path: "".into(),
        name: format!("<cadena>::{BYTES}"),
        func: |this, _, _, _| {
          let string = this.as_string();
          let list = string
            .as_bytes()
            .iter()
            .map(|b| Value::Byte(*b))
            .collect::<Vec<_>>();
          Ok(Value::Object(list.into()))
        },
        chunk: ChunkGroup::default().into(),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    SPLIT.into(),
    Value::Object(
      Function::Native {
        path: "".into(),
        name: format!("<cadena>::{SPLIT}"),
        func: |this, args, _, _| {
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
        chunk: ChunkGroup::default().into(),
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );

  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
