use crate::compiler::{BigUFloat, BigUInt, ChunkGroup, Function, Number, RealNumber, Value};

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
const PI_VAL: [u8; 107] = [
  48, 128, 129, 92, 194, 153, 239, 222, 150, 89, 57, 35, 170, 203, 111, 20, 83, 168, 65, 67, 213,
  228, 122, 88, 140, 194, 200, 12, 77, 115, 12, 178, 166, 121, 44, 210, 213, 11, 243, 122, 81, 117,
  170, 230, 212, 177, 116, 159, 0, 39, 217, 14, 94, 135, 104, 214, 228, 39, 216, 1, 17, 161, 49,
  132, 134, 58, 104, 252, 127, 189, 26, 16, 88, 50, 45, 28, 17, 19, 54, 163, 10, 48, 10, 157, 40,
  245, 187, 171, 222, 140, 185, 126, 130, 164, 236, 163, 240, 96, 15, 91, 93, 7, 219, 173, 128,
  172, 1,
];
const TAU_VAL: [u8; 107] = [
  97, 0, 3, 185, 132, 51, 223, 189, 45, 179, 114, 70, 84, 151, 223, 40, 166, 80, 131, 134, 170,
  201, 245, 176, 24, 133, 145, 25, 154, 230, 24, 100, 77, 243, 88, 164, 171, 23, 230, 245, 162,
  234, 84, 205, 169, 99, 233, 62, 1, 78, 178, 29, 188, 14, 209, 172, 201, 79, 176, 3, 34, 66, 99,
  8, 13, 117, 208, 248, 255, 122, 53, 32, 176, 100, 90, 56, 34, 38, 108, 70, 21, 96, 20, 58, 81,
  234, 119, 87, 189, 25, 115, 253, 4, 73, 217, 71, 225, 193, 30, 182, 186, 14, 182, 91, 1, 89, 3,
];
const EULER_VAL: [u8; 107] = [
  88, 168, 7, 30, 192, 221, 219, 235, 222, 171, 242, 3, 216, 8, 4, 209, 125, 17, 32, 22, 94, 252,
  172, 206, 142, 0, 102, 65, 163, 232, 22, 254, 225, 143, 143, 4, 81, 50, 160, 108, 1, 142, 181,
  167, 126, 227, 169, 134, 58, 54, 219, 241, 194, 222, 18, 115, 64, 101, 218, 213, 148, 198, 167,
  145, 103, 111, 94, 238, 106, 135, 154, 221, 94, 145, 196, 159, 184, 105, 242, 218, 117, 185, 26,
  6, 35, 100, 154, 80, 150, 225, 208, 73, 132, 185, 148, 213, 173, 11, 221, 4, 114, 9, 33, 181,
  195, 114, 1,
];

macro_rules! float {
  ($digits:expr) => {
    Value::Number(Number::Real(RealNumber::Float(
      false,
      BigUFloat::new(BigUInt::new($digits.to_vec()), 255),
    )))
  };
}

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
          let mut max = &Number::NegativeInfinity;
          for arg in &args {
            if arg.is_number() {
              let number = arg.as_number()?;
              if number > max {
                max = number;
              }
            } else {
              Err(format!("{MAX}: se esperaba un número"))?
            }
          }
          Ok(Value::Number(max.clone()))
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
          let mut min = &Number::Infinity;
          for arg in &args {
            if arg.is_number() {
              let number = arg.as_number()?;
              if number < min {
                min = number;
              }
            } else {
              Err(format!("{MIN}: se esperaba un número"))?
            }
          }
          Ok(Value::Number(min.clone()))
        },
        custom_data: ().into(),
      }
      .into(),
    ),
    true,
  );
  hashmap.set_instance_property(PI, float!(PI_VAL), true);
  hashmap.set_instance_property(TAU, float!(TAU_VAL), true);
  hashmap.set_instance_property(EULER, float!(EULER_VAL), true);
  Value::Object(crate::compiler::Object::Map(
    Default::default(),
    hashmap.into(),
  ))
}

#[test]
fn test() {
  const PI_STR: &str = "3.141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067982148086513282306647093844609550582231725359408128481117450284102701938521105559644622948954930381964428810975665933446128475648233786783165271201909145648";
  const TAU_STR: &str = "6.283185307179586476925286766559005768394338798750211641949889184615632812572417997256069650684234135964296173026564613294187689219101164463450718816256962234900568205403877042211119289245897909860763928857621951331866892256951296467573566330542403818291297";
  const EULER_STR: &str = "2.718281828459045235360287471352662497757247093699959574966967627724076630353547594571382178525166427427466391932003059921817413596629043572900334295260595630738132328627943490763233829880753195251019011573834187930702154089149934884167509244761460668082264";
  assert_eq!(
    PI_STR.parse::<BigUFloat>().unwrap(),
    BigUFloat::new(BigUInt::new(PI_VAL.to_vec()), 255)
  );
  assert_eq!(
    TAU_STR.parse::<BigUFloat>().unwrap(),
    BigUFloat::new(BigUInt::new(TAU_VAL.to_vec()), 255)
  );
  assert_eq!(
    EULER_STR.parse::<BigUFloat>().unwrap(),
    BigUFloat::new(BigUInt::new(EULER_VAL.to_vec()), 255)
  );
}
