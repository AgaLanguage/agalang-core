use std::rc::Rc;

use crate::{
  libraries, parser,
  runtime::{
    self,
    env::RefEnvironment,
    stack::RefStack,
    values::{
      self, complex, error_message, internal, primitive,
      traits::{self, AgalValuable as _, ToAgalValue as _},
      AgalValue,
    },
  },
  util,
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
  fn to_agal_string(
    &self,
    stack: RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[nativo fn {}]",
      self.name
    )))
  }
  fn to_agal_console(
    &self,
    stack: RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::CYAN),
    )
  }
  fn call(
    &self,
    stack: RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    (self.func)(args, stack, modules, this)
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(primitive::AgalBoolean::True)
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: parser::NodeOperator,
    right: values::DefaultRefAgalValue,
    modules: libraries::RefModules,
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
          type_error: parser::ErrorNames::TypeError,
          message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
          stack,
        }
        .to_result()
      }
    };
    match operator {
      parser::NodeOperator::Equal => {
        primitive::AgalBoolean::new(self.equals(&native_function)).to_result()
      }
      parser::NodeOperator::NotEqual => {
        primitive::AgalBoolean::new(!self.equals(&native_function)).to_result()
      }
      _ => internal::AgalThrow::Params {
        type_error: parser::ErrorNames::TypeError,
        message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
        stack,
      }
      .to_result(),
    }
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      complex::FUNCTION_CALL => modules
        .get_module(":proto/Funcion")
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
    Rc::as_ptr(&self.func) == Rc::as_ptr(&other.func)
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
impl traits::ToAgalValue for AgalNativeFunction {
  fn to_value(self) -> AgalValue {
    AgalInternal::NativeFunction(self).to_value()
  }
}
