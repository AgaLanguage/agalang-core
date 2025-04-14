use super::{
  internal,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalPrimitive, AgalValue,
};
use crate::{
  functions_names, libraries, parser,
  runtime::{
    self,
    values::{self, complex, error_message},
  },
  util,
};
pub const STRING_REPLACE: &str = "reemplaza";
pub const STRING_BYTES: &str = "bytes";
pub const STRING_SPLIT: &str = "partir";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgalChar(char);

impl AgalChar {
  pub fn new(value: char) -> Self {
    Self(value)
  }
  pub fn as_char(&self) -> char {
    self.0
  }
}

impl traits::ToAgalValue for AgalChar {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::Char(self).to_value()
  }
}
impl traits::AgalValuable for AgalChar {
  fn get_name(&self) -> String {
    "Caracter".to_string()
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(AgalString(vec![*self]))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    let char = self.as_char();
    Ok(AgalString::from_string(util::Color::BLUE.apply(
      &if char == '\'' {
        "\\'".to_string()
      } else {
        char.to_string()
      },
    )))
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    match self.as_char().to_digit(10) {
      Some(digit) => Ok(super::AgalNumber::from(digit as i32)),
      None => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_NUMBER.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalBoolean, internal::AgalThrow> {
    Ok(super::AgalBoolean::True)
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Primitive(p) = right.un_ref() {
      p.un_ref()
    } else {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_owned(),
        stack,
      });
    };
    match (operator, other) {
      (parser::NodeOperator::Equal, AgalPrimitive::Char(other)) => {
        super::AgalBoolean::new(self.equals(&other)).to_result()
      }
      (parser::NodeOperator::NotEqual, AgalPrimitive::Char(other)) => {
        super::AgalBoolean::new(!self.equals(&other)).to_result()
      }
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      functions_names::TO_AGAL_STRING => modules
        .get_module(":proto/Cadena")
        .ok_or_else(|| internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
          stack: stack.clone(),
        })?
        .get_instance_property(stack, key, modules),
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    self.as_char() == other.as_char()
  }

  fn less_than(&self, other: &Self) -> bool {
    (self.as_char() as u16) < (other.as_char() as u16)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgalString(Vec<AgalChar>);
impl AgalString {
  pub fn to_string(&self) -> String {
    self.0.iter().map(|c| c.0).collect()
  }
  pub fn from_string(value: String) -> Self {
    Self(value.chars().map(|c| AgalChar::new(c)).collect())
  }
  pub fn to_agal_chars(&self) -> Vec<AgalChar> {
    self.0.clone()
  }
  pub fn add_prev(&self, other: &str) -> Self {
    let mut new = vec![];
    new.extend(other.chars().map(|c| AgalChar::new(c)));
    new.extend(self.0.iter().map(|c| *c));
    Self(new)
  }
  pub fn add_post(&self, other: &str) -> Self {
    let mut new = vec![];
    new.extend(self.0.iter().map(|c| *c));
    new.extend(other.chars().map(|c| AgalChar::new(c)));
    Self(new)
  }
  pub fn add_sides(&self, pre: &str, post: &str) -> Self {
    let mut new = vec![];
    new.extend(pre.chars().map(|c| AgalChar::new(c)));
    new.extend(self.0.iter().map(|c| *c));
    new.extend(post.chars().map(|c| AgalChar::new(c)));
    Self(new)
  }
  pub fn set_color(&self, color: util::Color) -> Self {
    self.add_sides(color.as_str(), util::Color::RESET.as_str())
  }
}
impl traits::ToAgalValue for AgalString {
  fn to_value(self) -> AgalValue {
    AgalPrimitive::String(self).to_value()
  }
}
impl traits::AgalValuable for AgalString {
  fn get_name(&self) -> String {
    "Cadena".to_string()
  }
  fn as_string(&self) -> String {
    format!("[{} {}]", self.get_name(), self.to_string())
  }
  fn try_to_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<String, internal::AgalThrow> {
    Ok(self.to_string())
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.clone())
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<AgalString, internal::AgalThrow> {
    let string = self
      .try_to_string(stack, modules)?
      .replace("\n", "\\n")
      .replace("\r", "\\r")
      .replace("\t", "\\t")
      .replace("\0", "\\0");
    let string = if string.contains("'") && string.contains("\"") {
      format!("'{}'", string.replace("\'", "\\\'"))
    } else if string.contains("'") {
      format!("\"{}\"", string)
    } else {
      format!("'{}'", string)
    };
    Ok(AgalString::from_string(util::Color::BLUE.apply(&string)))
  }
  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalBoolean, internal::AgalThrow> {
    Ok(super::AgalBoolean::new(!self.0.is_empty()))
  }
  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    let clean_string = self.to_string();
    let clean_string = clean_string.trim_end_matches('0').trim_end_matches('.');
    let value = if clean_string.contains('.') {
      let value = clean_string.parse();
      match value {
        Ok(v) => Some(super::AgalNumber::Decimal(v)),
        Err(_) => None,
      }
    } else {
      let value = clean_string.parse();
      match value {
        Ok(v) => Some(super::AgalNumber::Integer(v)),
        Err(_) => None,
      }
    };
    value.ok_or_else(|| internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_owned(),
      stack,
    })
  }
  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<values::RefAgalValue<complex::AgalArray>, internal::AgalThrow> {
    Ok(complex::AgalArray::from(self).as_ref())
  }
  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Primitive(p) = right.un_ref() {
      p.un_ref()
    } else {
      return Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_owned(),
        stack,
      });
    };
    match (operator, other) {
      (parser::NodeOperator::Plus, AgalPrimitive::String(other)) => {
        self.add_post(&other.to_string()).to_result()
      }
      (parser::NodeOperator::Multiply, AgalPrimitive::Number(other)) => {
        AgalString::from_string(self.to_string().repeat(other.to_usize(stack)?)).to_result()
      }
      (parser::NodeOperator::Equal, AgalPrimitive::String(other)) => {
        super::AgalBoolean::new(self.equals(&other)).to_result()
      }
      (parser::NodeOperator::NotEqual, AgalPrimitive::String(other)) => {
        super::AgalBoolean::new(!self.equals(&other)).to_result()
      }
      _ => Err(internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }),
    }
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      functions_names::TO_AGAL_STRING | STRING_REPLACE | STRING_BYTES | STRING_SPLIT => modules
        .get_module(":proto/Cadena")
        .ok_or_else(|| internal::AgalThrow::Params {
          type_error: parser::ErrorNames::TypeError,
          message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
          stack: stack.clone(),
        })?
        .get_instance_property(stack, key, modules),
      "longitud" => super::AgalNumber::Integer(self.0.len() as i32).to_result(),
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::GET_INSTANCE_PROPERTY.to_owned(),
        stack,
      }
      .to_result(),
    }
  }

  fn equals(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }

  fn less_than(&self, other: &Self) -> bool {
    self.0.len() < other.0.len()
  }
}

impl<T: ToString> From<T> for AgalString {
  fn from(value: T) -> Self {
    Self::from_string(value.to_string())
  }
}
