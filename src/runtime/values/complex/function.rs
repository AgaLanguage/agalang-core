use parser::{
  ast::{NodeBlock, NodeIdentifier},
  util::{List, RefValue},
  KeywordsType,
};

use crate::runtime::{
  self,
  values::{
    internal, primitive,
    traits::{self, AgalValuable, ToAgalValue},
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
    Ok(
      self
        .to_agal_string()?
        .add_prev(&format!("\x1b[36m"))
        .add_post(&format!("\x1b[0m")),
    )
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
}
