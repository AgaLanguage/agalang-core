use parser::{
  ast::{NodeBlock, NodeIdentifier},
  util::{List, RefValue},
  KeywordsType,
};

use crate::{
  colors,
  runtime::{
    self, interpreter,
    values::{
      internal, primitive,
      traits::{self, AgalValuable, ToAgalValue},
    },
  },
};

#[derive(Clone, Debug)]
pub struct AgalFunction {
  name: String,
  is_async: bool,
  args: List<NodeIdentifier>,
  body: NodeBlock,
  env: runtime::RefEnvironment,
}

impl AgalFunction {
  pub fn new(
    name: String,
    is_async: bool,
    args: List<NodeIdentifier>,
    body: NodeBlock,
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
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(primitive::AgalString::from_string(format!(
      "[{} {}]",
      if self.is_async {
        format!(
          "{} {}",
          KeywordsType::Async.as_str(),
          KeywordsType::Function.as_str()
        )
      } else {
        KeywordsType::Function.to_string()
      },
      self.name
    )))
  }
  fn to_agal_console(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string(stack)?.set_color(colors::Color::CYAN))
  }
  async fn call(
    &self,
    stack: runtime::RefStack,
    this: crate::runtime::values::DefaultRefAgalValue,
    args: Vec<crate::runtime::values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
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
        &parser::ast::Node::Identifier(arg.clone()),
      );
    }
    let value = interpreter(self.body.clone().to_node().to_box(), stack, modules);
    if self.is_async {
      super::promise::AgalPromise::new(value).to_result()
    } else {
      value.await
    }
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: runtime::RefStack,
  ) -> Result<runtime::values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: runtime::RefStack,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: runtime::RefStack,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: runtime::RefStack,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    todo!()
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}
