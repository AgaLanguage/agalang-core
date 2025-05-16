use std::collections::HashMap;

use crate::{
  bytecode::value::{Function, Object, Value},
  util::Color,
};

pub const CONSOLE_LIB: &str = ":consola";
const DRAW: &str = "pintar";

fn inspect(value: &Value) -> String {
  match value {
    Value::Byte(_) => Color::Magenta,
    Value::Char(_) => Color::Blue,
    Value::False | Value::True | Value::Number(_) => Color::Yellow,
    Value::String(_) => Color::BrightGreen,
    Value::Never | Value::Null => Color::Gray,
    Value::Object(_) => Color::Cyan,
  }
  .apply(&value.as_string())
}

pub fn console_lib() -> Value {
  let mut hashmap = HashMap::new();

  hashmap.insert(
    DRAW.into(),
    Value::Object(Object::Function(Function::Native {
      name: format!("<{CONSOLE_LIB}>::{DRAW}"),
      path: format!("<{CONSOLE_LIB}>::{DRAW}"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let value = args
          .get(0)
          .ok_or_else(|| format!("{DRAW}: se esperaba 1 argumento y se recibieron 0"))?;
        match value {
          Value::Object(Object::Array(arr)) => {
            print!("[");
            let mut is_first = true;
            for item in arr.borrow().clone() {
              if is_first {
                is_first = false;
              } else {
                print!(", ")
              };
              print!("{}", inspect(&item));
            }
            println!("]");
          }
          item => println!("{}", inspect(item)),
        }
        Ok(Value::Never)
      },
    })),
  );
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
