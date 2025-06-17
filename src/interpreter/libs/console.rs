use std::collections::HashMap;

use crate::compiler::{Object, Value};
use crate::functions_names::CONSOLE;
use crate::interpreter::Thread;
use crate::util::{Color, OnError, OnSome};

pub const CONSOLE_LIB: &str = ":consola";
const DRAW: &str = "pinta";
const INSPECT: &str = "inspecciona";

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
    Value::Promise(_) => Color::Red,
    Value::Lazy(l) => return inspect_value(&l.get().on_some(|l| l.clone()).unwrap_or_default()),
  }
  .apply(&value.as_string())
}
fn inspect(value: &Value, thread: &Thread) -> String {
  match value {
    Value::Object(Object::Map(_, i)) => i
      .as_ref()
      .on_some_option(|i| i.get_instance_property(CONSOLE, thread))
      .on_some(|p| p.as_string())
      .unwrap_or_else(|| inspect_value(value)),
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
    Value::Iterator(iter) => format!("@{}", inspect(&iter.borrow(), thread)),
    Value::Ref(item) => format!("&{}", inspect(&item.borrow(), thread)),
    item => inspect_value(item),
  }
}

pub fn console_lib() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{CONSOLE_LIB}>"));

  hashmap.set_instance_property(
    DRAW.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{CONSOLE_LIB}>::{DRAW}"),
        path: format!("<{CONSOLE_LIB}>"),
        chunk: crate::compiler::ChunkGroup::default(),
        func: |_, args, thread| {
          for value in args.iter() {
            print!("{}", inspect(value, thread));
            print!(" ");
          }
          println!("");
          Ok(Value::Never)
        },
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    INSPECT.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{CONSOLE_LIB}>::{INSPECT}"),
        path: format!("<{CONSOLE_LIB}>"),
        chunk: crate::compiler::ChunkGroup::default(),
        func: |_, args, thread| {
          args
          .first()
          .on_some_option(|i|
            i.get_instance_property(CONSOLE, thread)
          ).on_some(|v|Value::String(v.as_string()))
          .on_error(|_|format!("{INSPECT}: se esperaba un número"))
        }
      }
      .into(),
    ),
    true,
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
