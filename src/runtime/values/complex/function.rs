use parser::{
  ast::{NodeBlock, NodeIdentifier},
  util::{List, RefValue},
  KeywordsType,
};

use crate::{
  colors,
  runtime::{
    self,
    values::{
      internal, primitive,
      traits::{self, AgalValuable, ToAgalValue},
    },
  },
};

#[derive(Clone)]
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
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow> {
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
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
  ) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string()?.set_color(colors::Color::CYAN))
  }
  async fn call(
    &self,
    stack: parser::util::RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: crate::runtime::values::DefaultRefAgalValue,
    args: Vec<crate::runtime::values::DefaultRefAgalValue>,
    modules: RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    let mut new_env = self.env.crate_child(false);
    new_env.set(crate::runtime::env::THIS_KEYWORD, this);
    for (i, arg) in self.args.iter().enumerate() {
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
    if self.is_async {
      let inner = super::promise::Promise::new(crate::runtime::interpreter::interpreter(
        parser::ast::Node::Block(self.body.clone()).to_box(),
        stack,
        new_env,
        modules,
      ));
      return Ok(super::promise::AgalPromise::new(inner).to_ref_value());
    }
    crate::runtime::interpreter::interpreter(
      parser::ast::Node::Block(self.body.clone()).to_box(),
      stack,
      new_env,
      modules,
    )
    .await
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalByte, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<runtime::values::RefAgalValue<super::AgalArray>, internal::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
  ) -> runtime::values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
    value: runtime::values::DefaultRefAgalValue,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    key: &str,
  ) -> Result<runtime::values::DefaultRefAgalValue, internal::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<runtime::Stack>,
  ) -> Result<primitive::AgalNumber, internal::AgalThrow> {
    todo!()
  }
}
