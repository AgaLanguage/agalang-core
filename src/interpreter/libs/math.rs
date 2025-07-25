use crate::compiler::{ChunkGroup, Function, Number, Value};

pub const LIB_NAME: &str = ":mate";
const CEIL: &str = "techo";
const FLOOR: &str = "suelo";
const ROUND: &str = "redondeo";
const MAX: &str = "max";
const MIN: &str = "min";
const PI: &str = "PI";
const EULER: &str = "E";
const TAU: &str = "TAU";
const IS_INFINITE: &str = "es_infinito";

pub fn lib_value() -> Value {
  let hashmap = crate::compiler::Instance::new(format!("<{LIB_NAME}>"));

  hashmap.set_instance_property(
    FLOOR,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{FLOOR}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let number = args
            .first()
            .ok_or_else(|| format!("{FLOOR}: se esperaba 1 argumento y se recibieron 0"))?;

          if number.is_number() {
            let number = number.as_number()?;
            Ok(Value::Number(number.floor()))
          } else {
            Err(format!("{FLOOR}: se esperaba un número"))
          }
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    IS_INFINITE,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{IS_INFINITE}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let number = args
            .first()
            .ok_or_else(|| format!("{IS_INFINITE}: se esperaba 1 argumento y se recibieron 0"))?;

          if number.is_number() {
            let number = number.as_number()?;
            Ok(Value::from(number.is_infinite()))
          } else {
            Err(format!("{IS_INFINITE}: se esperaba un número"))
          }
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    ROUND,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{ROUND}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let number = args
            .first()
            .ok_or_else(|| format!("{ROUND}: se esperaba 1 argumento y se recibieron 0"))?;

          if number.is_number() {
            let number = number.as_number()?;
            Ok(Value::Number(number.round()))
          } else {
            Err(format!("{ROUND}: se esperaba un número"))
          }
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    CEIL,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{CEIL}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let number = args
            .first()
            .ok_or_else(|| format!("{CEIL}: se esperaba 1 argumento y se recibieron 0"))?;

          if number.is_number() {
            let number = number.as_number()?;
            Ok(Value::Number(number.ceil()))
          } else {
            Err(format!("{CEIL}: se esperaba un número"))
          }
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    MAX,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{MAX}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let mut max = Number::NegativeInfinity;
          for arg in args {
            if arg.is_number() {
              let number = arg.as_number()?;
              if number > max {
                max = number;
              }
            } else {
              Err(format!("{MAX}: se esperaba un número"))?
            }
          }
          Ok(Value::Number(max))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(
    MIN,
    Value::Object(
      Function::Native {
        name: format!("<{LIB_NAME}>::{MIN}"),
        path: format!("<{LIB_NAME}>"),
        chunk: ChunkGroup::default().into(),
        func: |_, args, _, _| {
          let mut min = Number::Infinity;
          for arg in args {
            if arg.is_number() {
              let number = arg.as_number()?;
              if number < min {
                min = number;
              }
            } else {
              Err(format!("{MIN}: se esperaba un número"))?
            }
          }
          Ok(Value::Number(min))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(PI, Value::Number("3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679".parse::<Number>().unwrap()),true);
  hashmap.set_instance_property(TAU, Value::Number("6.2831853071795864769252867665590057683943387987502116419498891846156328125724179972560696506842341359".parse::<Number>().unwrap()),true);
  hashmap.set_instance_property(EULER, Value::Number("2.7182818284590452353602874713526624977572470936999595749669676277240766303535475945713821785251664274".parse::<Number>().unwrap()),true);
  Value::Object(crate::compiler::Object::Map(
    Default::default(),
    hashmap.into(),
  ))
}
