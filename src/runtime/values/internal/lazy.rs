use crate::{
  libraries,
  runtime::{
    self, interpreter,
    values::{
      self, primitive,
      traits::{self, AgalValuable as _, ToAgalValue},
      AgalValue, ResultAgalValue,
    },
    RefEnvironment, RefStack, Stack,
  },
};

use super::AgalInternal;

#[derive(Clone, Debug)]
pub struct AgalLazy {
  node: crate::parser::NodeExpressionMedicator,
  value: Option<ResultAgalValue>,
  stack: RefStack,
  modules: libraries::RefModules,
}
impl AgalLazy {
  pub fn new(
    node: crate::parser::NodeExpressionMedicator,
    stack: RefStack,
    modules: libraries::RefModules,
  ) -> Self {
    AgalLazy {
      node,
      value: None,
      stack,
      modules,
    }
  }
  pub fn get(&mut self) -> ResultAgalValue {
    if let Some(v) = &self.value {
      return v.clone();
    }
    let value = interpreter(
      self.node.expression.clone(),
      self.stack.clone(),
      self.modules.clone(),
    );
    self.value = Some(value.clone());

    value
  }
}

impl traits::AgalValuable for AgalLazy {
  fn get_name(&self) -> String {
    "Vago".to_string()
  }
  fn to_agal_string(
    &self,
    stack: runtime::RefStack,
    modules: libraries::RefModules,
  ) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<valor vago>".to_string(),
    ))
  }

  fn equals(&self, other: &Self) -> bool {
    false
  }

  fn less_than(&self, other: &Self) -> bool {
    false
  }
}
impl traits::ToAgalValue for AgalLazy {
  fn to_value(self) -> AgalValue {
    AgalInternal::Lazy(self).to_value()
  }
}
