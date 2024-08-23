use crate::runtime::{
    binary_operation_error, env::RefEnviroment, get_instance_property_error, AgalArray,
    AgalValuable, AgalValue, RefAgalValue, Stack,
};

use super::AgalBoolean;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgalByte(u8);
impl AgalByte {
    pub fn new(value: u8) -> AgalByte {
        AgalByte(value)
    }
    pub fn to_u8(&self) -> u8 {
        self.0
    }
}
impl AgalValuable for AgalByte {
    fn to_value(self) -> AgalValue {
        AgalValue::Byte(self)
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        _: RefEnviroment,
        operator: String,
        _other_: RefAgalValue,
    ) -> RefAgalValue {
        let _other: &AgalValue = &_other_.borrow();
        match _other {
            AgalValue::Byte(other) => {
                let boolean: bool = match operator.as_str() {
                    "+" => {
                        let a = self.0 as u16;
                        let b = other.0 as u16;
                        let c = a + b;
                        let byte1 = ((c >> 8) & 0xFF) as u8;
                        let byte2 = (c & 0xFF) as u8;
                        return AgalValue::Array(AgalArray::from_vec(vec![
                            AgalValue::Byte(AgalByte::new(byte1)).as_ref(),
                            AgalValue::Byte(AgalByte::new(byte2)).as_ref(),
                        ]))
                        .as_ref();
                    }
                    "-" => {
                        let a = self.0 as i16;
                        let b = other.0 as i16;
                        if b > a {
                            return binary_operation_error(
                                stack,
                                operator,
                                AgalValue::Byte(*self).as_ref(),
                                Some(_other_.clone()),
                            );
                        }
                        let c = a - b;
                        let byte1 = ((c >> 8) & 0xFF) as u8;
                        let byte2 = (c & 0xFF) as u8;
                        return AgalValue::Array(AgalArray::from_vec(vec![
                            AgalValue::Byte(AgalByte::new(byte1)).as_ref(),
                            AgalValue::Byte(AgalByte::new(byte2)).as_ref(),
                        ]))
                        .as_ref();
                    }
                    "*" => {
                        let a = self.0 as u16;
                        let b = other.0 as u16;
                        let c = a * b;
                        let byte1 = ((c >> 8) & 0xFF) as u8;
                        let byte2 = (c & 0xFF) as u8;
                        return AgalValue::Array(AgalArray::from_vec(vec![
                            AgalValue::Byte(AgalByte::new(byte1)).as_ref(),
                            AgalValue::Byte(AgalByte::new(byte2)).as_ref(),
                        ]))
                        .as_ref();
                    }
                    "/" => {
                        let a = self.0;
                        let b = other.0;
                        if b == 0 {
                            return binary_operation_error(
                                stack,
                                operator,
                                AgalValue::Byte(*self).as_ref(),
                                Some(_other_.clone()),
                            );
                        }
                        return AgalValue::Byte(AgalByte::new(a / b)).as_ref();
                    }
                    "%" => {
                        let a = self.0;
                        let b = other.0;
                        if b == 0 {
                            return binary_operation_error(
                                stack,
                                operator,
                                AgalValue::Byte(*self).as_ref(),
                                Some(_other_.clone()),
                            );
                        }
                        return AgalValue::Byte(AgalByte::new(a % b)).as_ref();
                    }
                    "==" => self.0 == other.0,
                    "!=" => self.0 != other.0,
                    "<" => self.0 < other.0,
                    "<=" => self.0 <= other.0,
                    ">" => self.0 > other.0,
                    ">=" => self.0 >= other.0,
                    "&&" => return (if self.0 == 0 {self} else {other}).to_value().as_ref(),
                    "||" => return (if self.0 != 0 {self} else {other}).to_value().as_ref(),
                    _ => {
                        return binary_operation_error(
                            stack,
                            operator,
                            AgalValue::Byte(*self).as_ref(),
                            Some(_other_.clone()),
                        )
                    }
                };
                AgalValue::Boolean(AgalBoolean::new(boolean)).as_ref()
            }
            _ => binary_operation_error(
                stack,
                operator,
                AgalValue::Byte(*self).as_ref(),
                Some(_other_.clone()),
            ),
        }
    }
    fn get_instance_property(self, stack: &Stack, env: RefEnviroment, key: String) -> RefAgalValue {
        let value = AgalValue::Byte(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn to_agal_string(
        self,
        _: &Stack,
        _: RefEnviroment,
    ) -> Result<super::AgalString, crate::runtime::AgalThrow> {
        Ok(super::AgalString::from_string(format!("{:08b}", self.0)))
    }
    fn to_agal_number(
        self,
        _: &Stack,
        _: RefEnviroment,
    ) -> Result<super::AgalNumber, crate::runtime::AgalThrow> {
        Ok(super::AgalNumber::new(self.0 as f64))
    }
}
