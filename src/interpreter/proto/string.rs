use crate::compiler::{ChunkGroup, Function, Object, Value};

const TYPE: &str = "<cadena>";

const REPLACE: &str = "reemplaza";
const REPEAT: &str = "repite";
const SPLIT: &str = "separa";
const BYTES: &str = "bytes";

pub fn prototype() -> Value {
  let hashmap = crate::compiler::Instance::new(TYPE.to_string());

  hashmap.set_instance_property(
    REPLACE,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{REPLACE}"),
        func: |this, args, thread, _| {
          let old = args.first();
          if old.is_none() {
            Err(format!(
              "{REPLACE}: se esperaban 2 argumentos y se recibieron 0"
            ))?;
          }
          let old = old.unwrap().as_string(thread);
          let new = args.get(1);
          if new.is_none() {
            Err(format!(
              "{REPLACE}: se esperaban 2 argumentos y se recibieron 1"
            ))?;
          }
          let new = new.unwrap().as_string(thread);
          let string = this.as_string(thread);
          let string = string.replace(&old, &new);
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
    REPEAT,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{REPEAT}"),
        func: |this, args, thread, _| {
          let count = match args.first() {
            None => Err(format!(
              "{REPEAT}: se esperaba 1 argumento y se recibieron 0"
            )),
            Some(count) => count.as_number()?.floor().into(),
          }?;
          if count == 0 {
            return Ok(Value::String("".to_string()));
          }
          let string = this.as_string(thread);
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
    BYTES,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{BYTES}"),
        func: |this, _, thread, _| {
          let string = this.as_string(thread);
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
    SPLIT,
    Value::Object(
      Function::Native {
        path: "".to_string(),
        name: format!("{TYPE}::{SPLIT}"),
        func: |this, args, thread, _| {
          let separator = args.first();
          if separator.is_none() {
            Err(format!(
              "{SPLIT}: se esperaba 1 argumento y se recibieron 0"
            ))?;
          }
          let separator = separator.unwrap().as_string(thread);
          let string = this.as_string(thread);
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

  Value::Object(Object::Map(Default::default(), hashmap.into()))
}
