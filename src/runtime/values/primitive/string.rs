use super::{
  internal,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalPrimitive, AgalValue,
};
use crate::{
  colors,
  runtime::{
    self,
    values::{complex::AgalArray, error_message, internal::AgalThrow, traits::ToAgalValue},
  },
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
  fn to_agal_string(&self) -> Result<AgalString, internal::AgalThrow> {
    Ok(AgalString(vec![*self]))
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    let char = self.as_char();
    Ok(AgalString::from_string(colors::Color::BLUE.apply(
      &if char == '\'' {
        "\\'".to_string()
      } else {
        char.to_string()
      },
    )))
  }

  fn get_keys(&self) -> Vec<String> {
    vec![]
  }

  fn to_agal_number(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<runtime::values::RefAgalValue<AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: runtime::values::DefaultRefAgalValue,
    args: Vec<runtime::values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgalString(Vec<AgalChar>);
impl AgalString {
  pub fn from_string(value: String) -> Self {
    Self(value.chars().map(|c| AgalChar::new(c)).collect())
  }
  pub fn to_string(&self) -> String {
    self.0.iter().map(|c| c.0).collect()
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
  pub fn set_color(&self, color: colors::Color) -> Self {
    self.add_sides(color.as_str(), colors::Color::RESET.as_str())
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
  fn try_to_string(&self) -> Result<String, internal::AgalThrow> {
    Ok(self.to_string())
  }
  fn to_agal_string(&self) -> Result<AgalString, internal::AgalThrow> {
    Ok(self.clone())
  }
  fn to_agal_console(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    self.to_agal_string()
  }
  fn to_agal_value(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<AgalString, internal::AgalThrow> {
    let string = self.try_to_string()?;
    let string = if string.contains("'") && string.contains("\"") {
      format!("'{}'", string.replace("\'", "\\\'"))
    } else if string.contains("'") {
      format!("\"{}\"", string)
    } else {
      format!("'{}'", string)
    };
    Ok(AgalString::from_string(colors::Color::BLUE.apply(&string)))
  }
  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::boolean::AgalBoolean, internal::AgalThrow> {
    Ok(super::boolean::AgalBoolean::new(!self.0.is_empty()))
  }
  fn get_keys(&self) -> Vec<String> {
    vec![]
  }
  fn to_agal_byte(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalByte, internal::AgalThrow> {
    Err(AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::TO_AGAL_BYTE.to_owned(),
      stack,
    })
  }
  fn to_agal_number(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<super::AgalNumber, internal::AgalThrow> {
    Err(AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::TO_AGAL_NUMBER.to_owned(),
      stack,
    })
  }
  fn to_agal_array(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
  ) -> Result<runtime::values::RefAgalValue<runtime::values::complex::AgalArray>, internal::AgalThrow>
  {
    Ok(AgalArray::from(self).as_ref())
  }
  fn binary_operation(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let other = if let AgalValue::Primitive(p) = right.un_ref() {
      p.un_ref()
    } else {
      return Err(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::TO_AGAL_STRING.to_owned(),
        stack,
      });
    };
    match (operator, other) {
      ("+", AgalPrimitive::String(other)) => self.add_post(&other.to_string()).to_result(),
      ("*", AgalPrimitive::Number(other)) => {
        AgalString::from_string(self.to_string().repeat(other.to_usize(stack)?)).to_result()
      }
      _ => Err(AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.clone().to_ref_value(), operator, right),
        stack,
      }),
    }
  }

  fn unary_back_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: runtime::values::DefaultRefAgalValue,
    args: Vec<runtime::values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: error_message::CALL.to_string(),
      stack,
    }
    .to_result()
  }
}
