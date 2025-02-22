use std::fmt::format;

use crate::runtime::values::traits::AgalValuable;

use super::DefaultRefAgalValue;

pub const SET_OBJECT_PROPERTY: &str = "No se pudo poner un valor";
pub const GET_OBJECT_PROPERTY: &str = "No se puede obtener la propiedad";
pub const GET_INSTANCE_PROPERTY: &str = "No se puede obtener la propiedad de instancia";

pub const INVALID_INSTANCE_PROPERTIES: &str = "No tengo esa propiedad de instancia";

pub const UNARY_OPERATOR: &str = "No se puede usar un operador unitario en este valor";
pub const UNARY_BACK_OPERATOR: &str = "No se puede usar un operador unitario trasero en este valor";
pub const CALL: &str = "No se puede invocar a este valor";

pub const TO_AGAL_CONSOLE: &str = "No se puede imprimir en consola";
pub const TO_AGAL_BYTE: &str = "No soy un byte";
pub const TO_AGAL_STRING: &str = "No soy una cadena";
pub const TO_AGAL_ARRAY: &str = "No soy una lista";
pub const TO_AGAL_NUMBER: &str = "No soy un numero";

pub const TYPE_ERROR_NUMBER: &str = "Se esperaba un número";

pub const ONLY_ONE_NUMBER_MULT: &str = "Solo se puede multiplicar un número";

pub const INVALID_OPERATOR: &str = "Operador invalido";

pub fn BINARY_OPERATION(
  left: DefaultRefAgalValue,
  operator: parser::ast::NodeOperator,
  right: DefaultRefAgalValue,
) -> String {
  format!(
    "No se puede hacer la operación '{} {} {}'",
    left.get_name(),
    operator,
    right.get_name()
  )
}
