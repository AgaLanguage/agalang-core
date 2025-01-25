use crate::{
  runtime::{
    delete_property_error, env::RefEnvironment, get_instance_property_error, get_property_error,
    set_property_error, unary_operation_error, AgalArray, AgalBoolean, AgalPrimitive, AgalString,
    AgalThrow, AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
  },
  Modules,
};

type BinNumber = f64;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct AgalNumber(BinNumber);
impl AgalNumber {
  pub fn new(value: BinNumber) -> AgalNumber {
    AgalNumber(value)
  }
  pub fn to_number(&self) -> BinNumber {
    self.0
  }
  pub fn multiply(&self, other: AgalNumber) -> AgalNumber {
    AgalNumber(self.0 * other.0)
  }
  pub fn neg(&self) -> AgalNumber {
    AgalNumber(-self.0)
  }
  pub fn flo(&self) -> AgalNumber {
    let str = format!("{}", self.0);
    let int_str: &str = str.split(".").collect::<Vec<_>>().get(0).unwrap_or(&"0");
    let int = int_str.parse().unwrap_or_default();

    AgalNumber(int)
  }
}
impl<'a> AgalValuable<'a> for AgalNumber {
  fn to_value(&self) -> &AgalValue {
    AgalPrimitive::Number(*self).to_value()
  }
  fn binary_operation(
    &self,
    stack: &Stack,
    _env: RefEnvironment,
    operator: &str,
    other: RefAgalValue,
  ) -> RefAgalValue {
    let other: &AgalValue = &other.borrow();
    match (other, operator) {
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "+") => {
        AgalNumber::new(self.0 + other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "-") => {
        AgalNumber::new(self.0 - other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "*") => {
        AgalNumber::new(self.0 * other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "/") => {
        if other.0 == 0f64 {
          return AgalThrow::Params {
            type_error: parser::internal::ErrorNames::MathError,
            message: "No se puede dividir por cero".to_string(),
            stack: Box::new(stack.clone()),
          }
          .to_ref_value();
        }
        AgalNumber::new(self.0 / other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "%") => {
        if other.0 == 0f64 {
          return AgalThrow::Params {
            type_error: parser::internal::ErrorNames::MathError,
            message: "No se puede dividir por cero".to_string(),
            stack: Box::new(stack.clone()),
          }
          .to_ref_value();
        }
        AgalNumber::new(self.0 % other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "==") => {
        AgalBoolean::new(self.0 == other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "!=") => {
        AgalBoolean::new(self.0 != other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "<") => {
        AgalBoolean::new(self.0 < other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "<=") => {
        AgalBoolean::new(self.0 <= other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), ">") => {
        AgalBoolean::new(self.0 > other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), ">=") => {
        AgalBoolean::new(self.0 >= other.0).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "&&") => {
        (if self.0 == 0f64 { self } else { other }).to_ref_value()
      }
      (AgalValue::Primitive(AgalPrimitive::Number(other)), "||") => {
        (if self.0 != 0f64 { self } else { other }).to_ref_value()
      }

      _ => AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: "No se puede operar con un valor no numerico".to_string(),
        stack: Box::new(stack.clone()),
      }
      .to_ref_value(),
    }
  }
  fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
    match operator {
      "?" => match self.to_agal_boolean(stack, env) {
        Ok(bool) => bool.to_ref_value(),
        Err(throw) => throw.to_ref_value(),
      },
      "!" => match self.to_agal_boolean(stack, env) {
        Ok(bool) => AgalBoolean::new(!bool.to_bool()).to_ref_value(),
        Err(throw) => throw.to_ref_value(),
      },
      "&" | "+" => self.to_ref_value(),
      "-" => self.neg().to_ref_value(),
      "~" => self.flo().to_ref_value(),
      _ => unary_operation_error(stack, operator, self.to_ref_value()),
    }
  }
  fn to_agal_number(&self, _: &Stack, _: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
    Ok(*self)
  }
  fn to_agal_boolean(&self, _: &Stack, _: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
    Ok(AgalBoolean::new(self.0 != 0f64))
  }
  fn to_agal_string(&self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string(self.0.to_string()))
  }
  fn to_agal_console(&self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
    Ok(AgalString::from_string(format!(
      "\x1b[33m{}\x1b[39m",
      self.0
    )))
  }
  fn get_instance_property(&self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    let value = self.to_value();
    get_instance_property_error(stack, env, key, value)
  }
  async fn call(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    _: RefAgalValue<'a>,
    list: Vec<RefAgalValue<'a>>,
    _: &Modules,
  ) -> RefAgalValue {
    let value = list.get(0);
    if value.is_none() {
      return self.to_ref_value();
    }
    let value = value.unwrap();
    let other = value.borrow().clone().to_agal_number(stack, env);
    if other.is_err() {
      return other.err().unwrap().to_value().as_ref();
    }
    let other = other.ok().unwrap();
    let number = self.multiply(other);
    number.to_ref_value()
  }

  fn get_keys(&self) -> Vec<String> {
    std::vec![]
  }

  fn get_length(&self) -> usize {
    0
  }

  fn to_agal_array(&self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
    Err(AgalThrow::Params {
      type_error: parser::internal::ErrorNames::CustomError("Error Iterable"),
      message: "El valor no es iterable".to_string(),
      stack: Box::new(stack.clone()),
    })
  }

  fn get_object_property(&self, stack: &Stack, env: RefEnvironment, key: String) -> RefAgalValue {
    get_property_error(stack, env, key)
  }

  fn set_object_property(
    &self,
    stack: &Stack,
    env: RefEnvironment,
    key: String,
    _value: RefAgalValue,
  ) -> RefAgalValue {
    set_property_error(stack, env, key, "No se puede asignar".to_string())
  }

  fn delete_object_property(self, stack: &Stack, env: RefEnvironment, key: String) {
    delete_property_error(stack, env, key);
  }
}
/*
#[derive(Clone)]
pub struct Number(Vec<u8>);

impl Number {
    pub fn from_str10(string: &str) -> Self {
        let bcd_list = {
            let string: String = {
                let len = string.len();
                if (len % 2) == 1 {
                    format!("0{}", string)
                } else {
                    string.to_string()
                }
            };
            let char_list: Vec<char> = string.chars().collect();
            let bytes_list = char_list.chunks(2).map(|c| {
                let first = c[0].to_digit(10);
                if first.is_none() {
                    panic!("'{}' not is a valid decimal number", c[1]);
                }
                let first = first.unwrap() as u8;
                let second = c[1].to_digit(10);
                if second.is_none() {
                    panic!("'{}' not is a valid decimal number", c[1]);
                }
                let second = second.unwrap() as u8;
                let mut byte = 0u8;
                byte += second;
                byte += first << 4;
                byte
            });
            let list: Vec<u8> = bytes_list.collect();
            list
        };
        Self::from_bcd_list(bcd_list)
    }
    pub fn from_bcd_list(bcd_list: Vec<u8>) -> Self {
        let mut bin_list: Vec<u8> = Vec::new();
        let len = bcd_list.len();
        let max = len - 1;
        for x in 0..len {
            let x = max - x;
            let value = bcd_list[x];
            let first = value >> 4;
            let second = (value << 4) >> 4;

            let first = first * 10;
            if x == max {
                let value = first + second;
                bin_list.push(value);
                continue;
            }
            let second = second * 10;
            let prev_bin = bin_list[0];
            let prev = (prev_bin >> 4) + second;
            let prev = (prev_bin & 0xFF) + (prev << 4);
            let second = second >> 4;
            let value = first + second;
            bin_list[0] = prev;
            bin_list.insert(0, value);
        }
        Self(bin_list)
    }
}

fn Vecu8toBin(vec: &Vec<u8>) -> String {
    let mut data = String::new();
    for x in vec {
        let string = format!("{:08b} ", x);
        data.push_str(&string);
    }
    data
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = Vecu8toBin(&self.0);
        write!(f, "{data}")
    }
}
*/
