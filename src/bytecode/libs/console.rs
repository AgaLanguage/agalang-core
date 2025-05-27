use std::collections::HashMap;

use crate::{
  bytecode::value::{Function, Object, Value},
  util::Color,
};

pub const CONSOLE_LIB: &str = ":consola";
const DRAW: &str = "pintar";

fn inspect_value(value: &Value) -> String {
  match value {
    Value::Byte(_) => Color::Magenta,
    Value::Char(_) => Color::Blue,
    Value::False | Value::True | Value::Number(_) => Color::Yellow,
    Value::String(_) => Color::BrightGreen,
    Value::Never | Value::Null => Color::Gray,
    Value::Object(_) => Color::Cyan,
    Value::Iterator(_) => Color::BrightCyan,
    Value::Ref(_) => Color::BrightBlue,
  }
  .apply(&value.as_string())
}
fn inspect(value: &Value) -> String {
  match value {
    Value::Object(Object::Array(arr)) => {
      let mut result = String::new();
      result.push_str("[");
      let mut is_first = true;
      for item in arr.borrow().clone() {
        if is_first {
          is_first = false;
        } else {
          result.push_str(", ");
        };
        result.push_str(&format!("{}", inspect_value(&item)));
      }
      result.push_str("]");
      result
    }
    Value::Iterator(iter) => format!("@{}", inspect(&iter.borrow())),
    Value::Ref(item) => format!("&{}", inspect(&item.borrow())),
    item => inspect_value(item),
  }
}

pub fn console_lib() -> Value {
  let mut hashmap = HashMap::new();

  hashmap.insert(
    DRAW.into(),
    Value::Object(
      Function::Native {
        name: format!("<{CONSOLE_LIB}>::{DRAW}"),
        path: format!("<{CONSOLE_LIB}>"),
        chunk: crate::bytecode::ChunkGroup::default(),
        func: |_, args| {
          for value in args.iter() {
            print!("{}", inspect(value));
            print!(" ");
          }
          println!("");
          Ok(Value::Never)
        },
      }
      .into(),
    ),
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
