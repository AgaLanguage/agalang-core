use parser::{ast::NodeExpressionMedicator, util::RefValue};

use crate::{
  runtime::{
    interpreter,
    values::{
      self, primitive,
      traits::{self, AgalValuable as _, ToAgalValue},
      AgalValue, ResultAgalValue,
    },
    RefEnvironment, RefStack, Stack,
  },
  Modules,
};

use super::AgalInternal;

#[derive(Clone, Debug)]
pub struct AgalLazy {
  node: NodeExpressionMedicator,
  value: Option<ResultAgalValue>,
  stack: RefStack,
  modules: RefValue<Modules>,
}
impl AgalLazy {
  pub fn new(node: NodeExpressionMedicator, stack: RefStack, modules: RefValue<Modules>) -> Self {
    AgalLazy {
      node,
      value: None,
      stack,
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
  fn to_agal_string(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<valor vago>".to_string(),
    ))
  }

  fn get_keys(&self) -> Vec<String> {
    todo!()
  }

  fn to_agal_byte(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalByte, super::AgalThrow> {
    todo!()
  }

  fn to_agal_boolean(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalBoolean, super::AgalThrow> {
    todo!()
  }

  fn to_agal_array(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<values::RefAgalValue<values::complex::AgalArray>, super::AgalThrow> {
    todo!()
  }

  fn binary_operation(
    &self,
    stack: crate::runtime::RefStack,
    operator: parser::ast::NodeOperator,
    right: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn get_object_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn set_object_property(
    &mut self,
    stack: crate::runtime::RefStack,
    key: &str,
    value: values::DefaultRefAgalValue,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn get_instance_property(
    &self,
    stack: crate::runtime::RefStack,
    key: &str,
  ) -> Result<values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  async fn call(
    &self,
    stack: crate::runtime::RefStack,
    this: values::DefaultRefAgalValue,
    args: Vec<values::DefaultRefAgalValue>,
    modules: parser::util::RefValue<crate::Modules>,
  ) -> Result<crate::runtime::values::DefaultRefAgalValue, super::AgalThrow> {
    todo!()
  }

  fn to_agal_number(
    &self,
    stack: crate::runtime::RefStack,
  ) -> Result<primitive::AgalNumber, super::AgalThrow> {
    todo!()
  }

  fn equals(&self, other: &Self) -> bool {
    todo!()
  }

  fn less_than(&self, other: &Self) -> bool {
    todo!()
  }
}
impl traits::ToAgalValue for AgalLazy {
  fn to_value(self) -> AgalValue {
    AgalInternal::Lazy(self).to_value()
  }
}
