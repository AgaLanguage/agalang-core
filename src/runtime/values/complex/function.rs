use crate::{
  libraries, parser,
  runtime::{
    self, async_interpreter, call_function_interpreter, interpreter,
    values::{
      error_message, internal, primitive,
      traits::{self, AgalValuable, ToAgalValue},
    },
  },
  util,
};
pub const FUNCTION_CALL: &str = "llamar";

#[derive(Clone, Debug)]
pub struct AgalFunction {
  name: String,
  is_async: bool,
  args: util::List<parser::NodeIdentifier>,
  body: parser::NodeBlock,
  env: runtime::RefEnvironment,
}

impl AgalFunction {
  pub fn new(
    name: String,
    is_async: bool,
    args: util::List<parser::NodeIdentifier>,
    body: parser::NodeBlock,
    env: runtime::RefEnvironment,
  ) -> Self {
    Self {
      name,
      is_async,
      args,
      body,
      env,
    }
  }
}

impl traits::ToAgalValue for AgalFunction {
  fn to_value(self) -> crate::runtime::values::AgalValue {
    super::AgalComplex::Function(self.as_ref()).to_value()
  }
}

impl traits::AgalValuable for AgalFunction {
  fn get_name(&self) -> String {
    "Función".to_string()
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[{} {}]",
      if self.is_async {
        format!(
          "{} {}",
          parser::KeywordsType::Async.as_str(),
          parser::KeywordsType::Function.as_str()
        )
      } else {
        parser::KeywordsType::Function.to_string()
      },
      self.name
    )))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(
      self
        .to_agal_string(stack, modules)?
        .set_color(util::Color::CYAN),
    )
  }
  fn call(
    &self,
    stack: runtime::RefStack,
    this: crate::runtime::values::DefaultRefAgalValue,
    args: Vec<crate::runtime::values::DefaultRefAgalValue>,
    modules: libraries::RefModules,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let mut new_env = self.env.crate_child(false);
    let stack = stack.with_env(new_env.clone());
    new_env.set(crate::runtime::env::THIS_KEYWORD, this);
    for (i, arg) in self.args.clone().enumerate() {
      let value = if i < args.len() {
        args[i].clone()
      } else {
        crate::runtime::values::AgalValue::Never.as_ref()
      };
      new_env.define(
        stack.clone(),
        &arg.name,
        value,
        false,
        &parser::Node::Identifier(arg.clone()),
      );
    }
    if self.is_async {
      super::promise::AgalPromise::new(call_function_interpreter(self.body.clone(), stack, modules))
        .to_result()
    } else {
      Ok(interpreter(self.body.clone().to_node().to_box(), stack, modules)?.into_return())
    }
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
    stack: runtime::RefStack,
    operator: parser::NodeOperator,
    right: runtime::values::DefaultRefAgalValue,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::ErrorNames::TypeError,
      message: error_message::BINARY_OPERATION(self.get_name(), operator, right.get_name()),
      stack,
    }
    .to_result()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
    modules: libraries::RefModules,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    match key {
      FUNCTION_CALL | crate::functions_names::TO_AGAL_STRING => modules
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
    (self.name == other.name)
      && (self.is_async == other.is_async)
      && (self.body == other.body)
      && (self.args == other.args)
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
