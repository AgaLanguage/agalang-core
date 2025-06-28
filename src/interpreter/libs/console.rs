use std::collections::HashMap;

use crate::compiler::{Object, Value};
use crate::functions_names::CONSOLE;
use crate::interpreter::Thread;
use crate::util::{Color, OnError, OnSome};

pub const LIB_NAME: &str = ":consola";
const DRAW: &str = "pinta";
const INSPECT: &str = "inspecciona";

fn inspect_value(value: &Value, thread: &Thread) -> String {
  match value {
    Value::Byte(_) => Color::Magenta,
    Value::Char(_) => Color::Blue,
    Value::False | Value::True | Value::Number(_) => Color::Yellow,
    Value::String(_) => Color::BrightGreen,
    Value::Never | Value::Null => Color::Gray,
    Value::Object(_) => Color::Cyan,
    Value::Iterator(_) => Color::BrightCyan,
    Value::Ref(_) => Color::BrightBlue,
    Value::Promise(_) => Color::DarkRed,
    Value::Lazy(l) => return inspect_value(&l.get().clone().unwrap_or_default(), thread),
  }
  .apply(&value.as_string(thread))
}
fn inspect(value: &Value, thread: &Thread) -> String {
  match value {
    Value::Object(Object::Map(_, i)) => i
      .read()
      .as_ref()
      .on_some_option(|i| i.get_instance_property(CONSOLE, thread))
      .map(|p| p.as_string(thread))
      .unwrap_or_else(|| inspect_value(value, thread)),
    Value::Object(Object::Array(arr)) => {
      let mut result = String::new();
      result.push_str("[");
      let mut is_first = true;
      for item in arr.read().clone() {
        if is_first {
          is_first = false;
        } else {
          result.push_str(", ");
        };
        result.push_str(&format!("{}", inspect_value(&item, thread)));
      }
      result.push_str("]");
      result
    }
    Value::Iterator(iter) => format!("@{}", inspect(&iter.read(), thread)),
    Value::Ref(item) => format!("&{}", inspect(&item.borrow(), thread)),
    item => inspect_value(item, thread),
  }
}

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    DRAW.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{DRAW}"),
        path: format!("<{LIB_NAME}>"),
        chunk: Default::default(),
        func: |_, args, thread, _| {
          for value in args.iter() {
            print!("{}", inspect(value, thread));
            print!(" ");
          }
          println!("");
          Ok(Value::Never)
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    INSPECT.into(),
    Value::Object(
      crate::compiler::Function::Native {
        name: format!("<{LIB_NAME}>::{INSPECT}"),
        path: format!("<{LIB_NAME}>"),
        chunk: Default::default(),
        func: |_, args, thread, _| {
          args
            .first()
            .map(|value| {
              value
                .get_instance_property(CONSOLE, thread)
                .unwrap_or_else(|| Value::String(inspect(value, thread)))
            })
            .on_error(|_| format!("{INSPECT}: se esperaba un valor para representar"))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
