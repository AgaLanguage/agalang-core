use parser::{ast::NodeExpressionMedicator, util::RefValue};

use crate::{
  runtime::{
    interpreter,
    values::{
      self, primitive,
      traits::{self, AgalValuable as _, ToAgalValue},
      AgalValue, ResultAgalValue,
    },
    RefEnvironment, Stack,
  },
  Modules,
};

use super::AgalInternal;

#[derive(Clone)]
pub struct AgalLazy {
  node: NodeExpressionMedicator,
  value: Option<ResultAgalValue>,
  stack: RefValue<Stack>,
  env: RefEnvironment,
  modules: RefValue<Modules>,
}
impl AgalLazy {
  pub fn new(
    node: NodeExpressionMedicator,
    stack: RefValue<Stack>,
    env: RefEnvironment,
    modules: RefValue<Modules>,
  ) -> Self {
    AgalLazy {
      node,
      value: None,
      stack,
      env,
      modules,
    }
  }
  pub async fn get(&mut self) -> ResultAgalValue {
    if let Some(v) = &self.value {
      return v.clone();
    }
    self.value = Some(AgalValue::Never.to_result());
    self.value = Some(
      interpreter(
        self.node.expression.clone(),
        self.stack.clone(),
        self.env.clone(),
        self.modules.clone(),
      )
      .await,
    );

    self.value.clone().unwrap()
  }
}

impl traits::AgalValuable for AgalLazy {
  fn get_name(&self) -> String {
    "Vago".to_string()
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<valor vago>".to_string(),
    ))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalByte, super::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalBoolean, super::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, super::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn unary_back_operator(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn unary_operator(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    operator: &str,
  ) -> values::ResultAgalValue {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: parser::util::RefValue<crate::runtime::Stack>,
    env: crate::runtime::RefEnvironment,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: RefValue<crate::runtime::Stack>,
  ) -> Result<primitive::AgalNumber, super::AgalThrow> {
    todo!()
  }
}
impl traits::ToAgalValue for AgalLazy {
  fn to_value(self) -> AgalValue {
    AgalInternal::Lazy(self).to_value()
  }
}
