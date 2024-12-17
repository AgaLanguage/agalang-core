use crate::runtime::{
    binary_operation_error, env::RefEnvironment, get_instance_property_error,
    unary_operation_error, AgalArray, AgalBoolean, AgalPrimitive, AgalString, AgalThrow,
    AgalValuable, AgalValuableManager, AgalValue, RefAgalValue, Stack,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalByte(u8, bool);
impl AgalByte {
    pub fn new(value: u8) -> AgalByte {
        AgalByte(value, false)
    }
    pub fn to_u8(&self) -> u8 {
        self.0
    }
}
impl<'a> AgalValuable<'a> for AgalByte {
    fn to_value(&self) -> &AgalValue {
        AgalPrimitive::Byte(*self).to_value()
    }
    fn to_agal_console(&self, _: &Stack, _: RefEnvironment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!(
            "\x1b[94m0by{:08b}\x1b[39m",
            self.0
        )))
    }
    fn to_agal_byte(&self, _: &Stack) -> Result<AgalByte, AgalThrow> {
        Ok(*self)
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        _: RefEnvironment,
        operator: &str,
        _other_: RefAgalValue,
    ) -> RefAgalValue {
        let _other: &AgalValue = &_other_.borrow();
        match (_other, operator) {
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "+") => {
                let a = self.0 as u16;
                let b = other.0 as u16;
                let c = a + b;
                let byte1 = ((c >> 8) & 0xFF) as u8;
                let byte2 = (c & 0xFF) as u8;
                AgalArray::from_vec(vec![
                    AgalByte::new(byte1).to_ref_value(),
                    AgalByte::new(byte2).to_ref_value(),
                ])
                .to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "-") => {
                let a = self.0 as i16;
                let b = other.0 as i16;
                if b > a {
                    return binary_operation_error(
                        stack,
                        operator,
                        (*self).to_ref_value(),
                        _other_.clone(),
                    );
                }
                let c = a - b;
                let byte1 = ((c >> 8) & 0xFF) as u8;
                let byte2 = (c & 0xFF) as u8;
                AgalArray::from_vec(vec![
                    AgalByte::new(byte1).to_ref_value(),
                    AgalByte::new(byte2).to_ref_value(),
                ])
                .to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "*") => {
                let a = self.0 as u16;
                let b = other.0 as u16;
                let c = a * b;
                let byte1 = ((c >> 8) & 0xFF) as u8;
                let byte2 = (c & 0xFF) as u8;
                AgalArray::from_vec(vec![
                    AgalByte::new(byte1).to_ref_value(),
                    AgalByte::new(byte2).to_ref_value(),
                ])
                .to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "/") => {
                let a = self.0;
                let b = other.0;
                if b == 0 {
                    return binary_operation_error(
                        stack,
                        operator,
                        (*self).to_ref_value(),
                        _other_.clone(),
                    );
                }
                AgalByte::new(a / b).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "%") => {
                let a = self.0;
                let b = other.0;
                if b == 0 {
                    return binary_operation_error(
                        stack,
                        operator,
                        (*self).to_ref_value(),
                        _other_.clone(),
                    );
                }
                AgalByte::new(a % b).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "==") => {
                AgalBoolean::new(self.0 == other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "!=") => {
                AgalBoolean::new(self.0 != other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "<") => {
                AgalBoolean::new(self.0 < other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "<=") => {
                AgalBoolean::new(self.0 <= other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), ">") => {
                AgalBoolean::new(self.0 > other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), ">=") => {
                AgalBoolean::new(self.0 >= other.0).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "&&") => {
                (if self.0 == 0 { self } else { other }).to_ref_value()
            }
            (AgalValue::Primitive(AgalPrimitive::Byte(other)), "||") => {
                (if self.0 != 0 { self } else { other }).to_ref_value()
            }
            _ => binary_operation_error(stack, operator, (*self).to_ref_value(), _other_.clone()),
        }
    }
    fn unary_operator(&self, stack: &Stack, env: RefEnvironment, operator: &str) -> RefAgalValue {
        match operator {
            "?" => match self.to_agal_boolean(stack, env) {
                Ok(bool) => bool.to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "!" => match self.to_agal_boolean(stack, env) {
                Ok(bool) => AgalBoolean::new(!bool.to_bool()).to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "+" => match self.to_agal_number(stack, env) {
                Ok(num) => num.to_ref_value(),
                Err(throw) => throw.to_ref_value(),
            },
            "&" => AgalByte(self.0, false).to_ref_value(),
            _ => unary_operation_error(stack, operator, self.to_ref_value()),
        }
    }
    fn get_instance_property(
        &self,
        stack: &Stack,
        env: RefEnvironment,
        key: String,
    ) -> RefAgalValue {
        let value = self.to_value();
        get_instance_property_error(stack, env, key, value)
    }
    fn to_agal_string(
        &self,
        _: &Stack,
        _: RefEnvironment,
    ) -> Result<super::AgalString, crate::runtime::AgalThrow> {
        Ok(super::AgalString::from_string(format!("{:08b}", self.0)))
    }
    fn to_agal_number(
        &self,
        _: &Stack,
        _: RefEnvironment,
    ) -> Result<super::AgalNumber, crate::runtime::AgalThrow> {
        Ok(super::AgalNumber::new(self.0 as f64))
    }
}
