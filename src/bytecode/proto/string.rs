use std::collections::HashMap;

use crate::bytecode::{
  value::{Function, Instance, Object, Value},
  ChunkGroup,
};

const REPLACE: &str = "remplaza";
const SPLIT: &str = "separa";
const BYTES: &str = "bytes";

pub fn string_proto() -> Value {
  let hashmap = Instance::new(format!("<cadena>"));

  hashmap.set_instance_property(
    REPLACE.into(),
    Value::Object(Function::Native {
      path: "".into(),
      name: format!("<cadena>::{REPLACE}"),
      func: |this, args, _| {
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
    }.into()),
  );
  hashmap.set_public_property(REPLACE, true);

  hashmap.set_instance_property(
    BYTES.into(),
    Value::Object(Function::Native {
      path: "".into(),
      name: format!("<cadena>::{BYTES}"),
      func: |this, _, _| {
        let string = this.as_string();
        let list = string
          .as_bytes()
          .iter()
          .map(|b| Value::Byte(*b))
          .collect::<Vec<_>>();
        Ok(Value::Object(list.into()))
      },
      chunk: ChunkGroup::default(),
    }.into()),
  );
  hashmap.set_public_property(BYTES, true);

  hashmap.set_instance_property(
    SPLIT.into(),
    Value::Object(Function::Native {
      path: "".into(),
      name: format!("<cadena>::{SPLIT}"),
      func: |this, args, _| {
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
    }.into()),
  );
  hashmap.set_public_property(SPLIT, true);

  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
