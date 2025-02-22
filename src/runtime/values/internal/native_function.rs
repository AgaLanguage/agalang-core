use std::rc::Rc;

use crate::{
  colors, libraries,
  runtime::{
    env::RefEnvironment,
    stack::RefStack,
    values::{
      self, error_message, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
};

use super::AgalInternal;

#[derive(Clone)]
pub struct AgalNativeFunction {
  pub name: String,
  pub func: Rc<
    dyn Fn(
      Vec<values::DefaultRefAgalValue>,
      RefStack,
      libraries::RefModules,
      values::DefaultRefAgalValue,
    ) -> Result<values::DefaultRefAgalValue, super::AgalThrow>,
  >,
}
impl std::fmt::Debug for AgalNativeFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "NativeFunction({})", self.name)
  }
}
impl traits::AgalValuable for AgalNativeFunction {
  fn get_name(&self) -> String {
    "<Función nativa>".to_string()
  }
  fn to_agal_string(&self, stack: RefStack) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[nativo fn {}]",
      self.name
    )))
  }
  fn to_agal_console(&self, stack: RefStack) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::CYAN))
  }
  async fn call(
    &self,
    stack: RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    (self.func)(args, stack, modules, this)
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    let native_function = match right.un_ref() {
      AgalValue::Internal(i) => match i.un_ref() {
        AgalInternal::NativeFunction(nf) => Some(nf),
        _ => None,
      },
      _ => None,
    };
    let native_function = match native_function {
      Some(nf) => nf,
      None => {
        return internal::AgalThrow::Params {
          type_error: parser::internal::ErrorNames::TypeError,
          message: error_message::BINARY_OPERATION(self.clone().to_ref_value(), operator, right.clone()),
          stack,
        }
        .to_result()
      }
    };
    match operator {
      parser::ast::NodeOperator::Equal => primitive::AgalBoolean::new(self.equals(&native_function)).to_result(),
      parser::ast::NodeOperator::NotEqual => primitive::AgalBoolean::new(!self.equals(&native_function)).to_result(),
      _ => internal::AgalThrow::Params {
        type_error: parser::internal::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.clone().to_ref_value(), operator, right.clone()),
        stack,
      }
      .to_result(),
    }
  }

  fn get_object_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: crate::runtime::RefStack,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    Rc::as_ptr(&self.func) == Rc::as_ptr(&other.func)
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}
impl traits::ToAgalValue for AgalNativeFunction {
  fn to_value(self) -> AgalValue {
    AgalInternal::NativeFunction(self).to_value()
  }
}
