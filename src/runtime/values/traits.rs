use parser::util::RefValue;

use crate::{
  runtime,
  Modules,
};

use super::{
  complex::AgalArray, internal, primitive, AgalValue, DefaultRefAgalValue, RefAgalValue, ResultAgalValue
};

pub trait ToAgalValue: AgalValuable {
  fn to_value(self) -> AgalValue;
  fn to_ref_value(self) -> DefaultRefAgalValue
  where
    Self: Sized,
  {
    self.to_value().as_ref()
  }
  fn to_result(self) -> Result<DefaultRefAgalValue, internal::AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_ref_value())
  }
}
pub trait AgalValuable {
  fn as_ref(self) -> RefAgalValue<Self>
  where
    Self: Sized + ToAgalValue,
  {
    RefAgalValue::new(self)
  }
  fn get_keys(&self) -> Vec<String> {
    vec![]
  }
  fn try_to_string(&self) -> Result<String,internal:: AgalThrow>
  where
    Self: Sized,
  {
    Ok(self.to_agal_string()?.to_string())
  }
  fn to_agal_string(&self) -> Result<primitive::AgalString, internal::AgalThrow>;
  fn to_agal_byte(&self, stack: RefValue<runtime::Stack>) -> Result<primitive::AgalByte, internal::AgalThrow> {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No soy un byte".to_string(),
      stack,
    })
  }
  fn to_agal_boolean(&self, stack: RefValue<runtime::Stack>) -> Result<primitive::AgalBoolean, internal::AgalThrow> {
    Ok(primitive::AgalBoolean::True)
  }
  fn to_agal_console(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment) -> Result<primitive::AgalString, internal::AgalThrow> {
    Ok(self.to_agal_string()?.add_prev("\x1b[35m").add_post("\x1b[0m"))
  }
  fn to_agal_value(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment) -> Result<primitive::AgalString, internal::AgalThrow> {
    self.to_agal_console(stack, env)
  }
  fn to_agal_array(&self, stack: RefValue<runtime::Stack>) -> Result<RefAgalValue<AgalArray>, internal::AgalThrow> {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No soy un array".to_string(),
      stack,
    })
  }
  fn binary_operation(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    operator: &str,
    right: DefaultRefAgalValue,
  ) -> Result<DefaultRefAgalValue, internal::AgalThrow> {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No puedo ser operado".to_string(),
      stack,
    })
  }
  fn unary_back_operator(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment, operator: &str) -> ResultAgalValue {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No puedo ser operado por detras".to_string(),
      stack,
    })
  }
  fn unary_operator(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment, operator: &str) -> ResultAgalValue {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No puedo ser operado por detras".to_string(),
      stack,
    })
  }
  fn get_object_property(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment, key: &str)-> Result<DefaultRefAgalValue, internal::AgalThrow> {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No tengo propiedades de instancia".to_string(),
      stack,
    })
  }
  fn set_object_property(&mut self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment, key: &str, value:DefaultRefAgalValue)-> Result<DefaultRefAgalValue, internal::AgalThrow> {
    Err(internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No puedes modificar mis propiedades".to_string(),
      stack,
    })
  }
  fn get_instance_property(&self, stack: RefValue<runtime::Stack>, env: runtime::RefEnvironment, key: &str)-> Result<DefaultRefAgalValue, internal::AgalThrow> {
    internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No tengo propiedades de instancia".to_string(),
      stack,
    }.to_result()
  }
  async fn call(
    &self,
    stack: RefValue<runtime::Stack>,
    env: runtime::RefEnvironment,
    this: DefaultRefAgalValue,
    args: Vec<DefaultRefAgalValue>,
    modules: RefValue<Modules>,
  ) -> Result<
    crate::runtime::values::DefaultRefAgalValue,
    internal::AgalThrow,
  > {
    internal::AgalThrow::Params {
      type_error: parser::internal::ErrorNames::TypeError,
      message: "No soy una funcion".to_string(),
      stack,
    }.to_result()
  }
}
