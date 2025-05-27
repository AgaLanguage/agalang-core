use std::collections::HashMap;

use crate::bytecode::value::{Function, Number, Object, Value};

pub const MATH_LIB: &str = ":mate";
const CEIL: &str = "techo";
const FLOOR: &str = "suelo";
const ROUND: &str = "redondeo";
const MAX: &str = "max";
const MIN: &str = "min";
const PI: &str = "PI";
const EULER: &str = "E";
const TAU: &str = "TAU";
const IS_INFINITE: &str = "esInfinito";

pub fn math_lib() -> Value {
  let mut hashmap = HashMap::new();

  hashmap.insert(
    FLOOR.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{FLOOR}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let number = args
          .get(0)
          .ok_or_else(|| format!("{FLOOR}: se esperaba 1 argumento y se recibieron 0"))?;

        if number.is_number() {
          let number = number.as_number();
          Ok(Value::Number(number.floor()))
        } else {
          Err(format!("{FLOOR}: se esperaba un número"))
        }
      },
    }.into()),
  );
    hashmap.insert(
    IS_INFINITE.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{IS_INFINITE}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let number = args
          .get(0)
          .ok_or_else(|| format!("{IS_INFINITE}: se esperaba 1 argumento y se recibieron 0"))?;

        if number.is_number() {
          let number = number.as_number();
          Ok(Value::from(number.is_infinite()))
        } else {
          Err(format!("{IS_INFINITE}: se esperaba un número"))
        }
      },
    }.into()),
  );
  hashmap.insert(
    ROUND.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{ROUND}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let number = args
          .get(0)
          .ok_or_else(|| format!("{ROUND}: se esperaba 1 argumento y se recibieron 0"))?;

        if number.is_number() {
          let number = number.as_number();
          Ok(Value::Number(number.round()))
        } else {
          Err(format!("{ROUND}: se esperaba un número"))
        }
      },
    }.into()),
  );
  hashmap.insert(
    CEIL.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{CEIL}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let number = args
          .get(0)
          .ok_or_else(|| format!("{CEIL}: se esperaba 1 argumento y se recibieron 0"))?;

        if number.is_number() {
          let number = number.as_number();
          Ok(Value::Number(number.ceil()))
        } else {
          Err(format!("{CEIL}: se esperaba un número"))
        }
      },
    }.into()),
  );
  hashmap.insert(
    MAX.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{MAX}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let mut max = Number::NegativeInfinity;
        for arg in args {
          if arg.is_number() {
            let number = arg.as_number();
            if number > max {
              max = number;
            }
          } else {
            return Err(format!("{MAX}: se esperaba un número"));
          }
        }
        Ok(Value::Number(max))
      },
    }.into()),
  );
  hashmap.insert(
    MIN.into(),
    Value::Object(Function::Native {
      name: format!("<{MATH_LIB}>::{MIN}"),
      path: format!("<{MATH_LIB}>"),
      chunk: crate::bytecode::ChunkGroup::default(),
      func: |_, args| {
        let mut min = Number::Infinity;
        for arg in args {
          if arg.is_number() {
            let number = arg.as_number();
            if number < min {
              min = number;
            }
          } else {
            return Err(format!("{MIN}: se esperaba un número"));
          }
        }
        Ok(Value::Number(min))
      },
    }.into()),
  );
  hashmap.insert(PI.into(), Value::Number("3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679".parse::<Number>().unwrap()));
  hashmap.insert(TAU.into(), Value::Number("6.2831853071795864769252867665590057683943387987502116419498891846156328125724179972560696506842341359".parse::<Number>().unwrap()));
  hashmap.insert(EULER.into(), Value::Number("2.7182818284590452353602874713526624977572470936999595749669676277240766303535475945713821785251664274".parse::<Number>().unwrap()));
  Value::Object(Object::Map(HashMap::new().into(), hashmap.into()))
}
