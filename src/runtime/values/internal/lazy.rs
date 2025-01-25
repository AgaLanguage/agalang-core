use parser::ast::NodeExpressionMedicator;

use crate::runtime::values::{
  self, primitive,
  traits::{self, AgalValuable as _, ToAgalValue as _},
  AgalValue,
};

use super::AgalInternal;

#[derive(Clone)]
pub struct AgalLazy {
  node: NodeExpressionMedicator,
  value: Option<values::DefaultRefAgalValue>,
}
impl AgalLazy {
  pub fn new(node: NodeExpressionMedicator) -> Self {
    AgalLazy { node, value: None }
  }
}

impl traits::AgalValuable for AgalLazy {
  fn to_agal_string(&self) -> Result<primitive::AgalString, super::AgalThrow> {
    Ok(primitive::AgalString::from_string(
      "<valor vago>".to_string(),
    ))
  }
}
impl traits::ToAgalValue for AgalLazy {
  fn to_value(self) -> AgalValue {
    AgalInternal::Lazy(self).to_value()
  }
}
