use crate::runtime::{
    env::{RefEnvironment, FALSE_KEYWORD, TRUE_KEYWORD},
    get_instance_property_error, unary_operation_error, AgalNumber, AgalPrimitive, AgalString,
    AgalThrow, AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalBoolean(bool);
impl AgalBoolean {
    pub fn new(value: bool) -> AgalBoolean {
        AgalBoolean(value)
    }
    pub fn to_bool(&self) -> bool {
        self.0
    }
}
fn bool_to_str(value: bool) -> String {
    let data = if value { TRUE_KEYWORD } else { FALSE_KEYWORD };
    data.to_string()
}
impl AgalValuable for AgalBoolean {
    fn to_value(self) -> AgalValue {
        AgalPrimitive::Boolean(self).to_value()
    }
    fn to_agal_boolean(self, _: &Stack, _: RefEnvironment) -> Result<AgalBoolean, AgalThrow> {
        Ok(self)
    }
    fn to_agal_string(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(bool_to_str(self.0)))
    }
    fn to_agal_number(self, _: &Stack, _: RefEnvironment) -> Result<AgalNumber, AgalThrow> {
        Ok(AgalNumber::new(if self.0 { 1f64 } else { 0f64 }))
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        _env: RefEnvironment,
        operator: &str,
        other: RefAgalValue,
    ) -> RefAgalValue {
        let other: &AgalValue = &other.borrow();
        match other {
            AgalValue::Primitive(AgalPrimitive::Boolean(other)) => {
                let boolean = match operator {
                    "&&" => AgalBoolean::new(self.0 && other.0),
                    "||" => AgalBoolean::new(self.0 || other.0),
                    "==" => AgalBoolean::new(self.0 == other.0),
                    "!=" => AgalBoolean::new(self.0 != other.0),
                    _ => {
                        return AgalThrow::Params {
                            type_error: parser::internal::ErrorNames::TypeError,
                            message: format!("Operador {} no soportado", operator),
                            stack: Box::new(stack.clone()),
                        }
                        .to_ref_value();
                    }
                };
                AgalPrimitive::Boolean(boolean).to_ref_value()
            }
            _ => AgalThrow::Params {
                type_error: parser::internal::ErrorNames::TypeError,
                message: "No se puede operar con un valor no booleano".to_string(),
                stack: Box::new(stack.clone()),
            }
            .to_ref_value(),
        }
    }
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match operator {
            "&" | "?" => self.to_ref_value(),
            "!" => AgalBoolean::new(!self.to_bool()).to_ref_value(),
            "+" => match self.to_agal_number(stack, env) {
                Ok(num) => num.to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            _ => unary_operation_error(stack, operator, self.to_ref_value()),
        }
    }
    fn to_agal_console(self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[33m{}\x1b[39m",
            bool_to_str(self.0)
        )))
    }
    fn get_instance_property(
        self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let value = self.to_value();
        get_instance_property_error(stack, env, key, value)
    }
}
