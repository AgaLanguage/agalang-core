use crate::runtime::{binary_operation_error, get_instance_property_error, AgalArray, AgalValuable, AgalValue, Enviroment, RefAgalValue, Stack};

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
        _: &Enviroment,
        operator: String,
        _other_: RefAgalValue,
    ) -> RefAgalValue {
        let _other: &AgalValue = &_other_.borrow();
        match _other {
            AgalValue::Byte(other) => match operator.as_str() {
                "+" => {
                    let a = self.0 as u16;
                    let b = other.0 as u16;
                    let c = a + b;
                    let byte1 = ((c >> 8) & 0xFF) as u8;
                    let byte2 = (c & 0xFF) as u8;
                    AgalValue::Array(AgalArray::from_vec(vec![
                        AgalValue::Byte(AgalByte::new(byte1)).to_ref(),
                        AgalValue::Byte(AgalByte::new(byte2)).to_ref(),
                    ])).to_ref()
                }
                "-" => {
                    let a = self.0 as i16;
                    let b = other.0 as i16;
                    if b > a {
                        return binary_operation_error(
                            stack,
                            operator,
                            AgalValue::Byte(*self).to_ref(),
                            Some(_other_.clone()),
                        );
                    }
                    let c = a - b;
                    let byte1 = ((c >> 8) & 0xFF) as u8;
                    let byte2 = (c & 0xFF) as u8;
                    AgalValue::Array(AgalArray::from_vec(vec![
                        AgalValue::Byte(AgalByte::new(byte1)).to_ref(),
                        AgalValue::Byte(AgalByte::new(byte2)).to_ref(),
                    ])).to_ref()
                }
                "*" => {
                    let a = self.0 as u16;
                    let b = other.0 as u16;
                    let c = a * b;
                    let byte1 = ((c >> 8) & 0xFF) as u8;
                    let byte2 = (c & 0xFF) as u8;
                    AgalValue::Array(AgalArray::from_vec(vec![
                        AgalValue::Byte(AgalByte::new(byte1)).to_ref(),
                        AgalValue::Byte(AgalByte::new(byte2)).to_ref(),
                    ])).to_ref()
                }
                "/" => {
                    let a = self.0;
                    let b = other.0;
                    if b == 0 {
                        return binary_operation_error(
                            stack,
                            operator,
                            AgalValue::Byte(*self).to_ref(),
                            Some(_other_.clone()),
                        );
                    }
                    AgalValue::Byte(AgalByte::new(a / b)).to_ref()
                }
                "%" => {
                    let a = self.0;
                    let b = other.0;
                    if b == 0 {
                        return binary_operation_error(
                            stack,
                            operator,
                            AgalValue::Byte(*self).to_ref(),
                            Some(_other_.clone()),
                        );
                    }
                    AgalValue::Byte(AgalByte::new(a % b)).to_ref()
                }
                _ => binary_operation_error(stack, operator, AgalValue::Byte(*self).to_ref(), Some(_other_.clone())),
            },
            _ => binary_operation_error(stack, operator, AgalValue::Byte(*self).to_ref(), Some(_other_.clone())),
        }
    }
    fn get_instance_property(self, stack: &Stack, env: &Enviroment, key: String) -> RefAgalValue {
        let value = AgalValue::Byte(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn to_agal_string(self, _: &Stack, _: &Enviroment) -> Result<super::AgalString, crate::runtime::AgalThrow> {
        Ok(super::AgalString::from_string(format!("{:08b}", self.0)))
    }
    fn to_agal_number(self, _: &Stack, _: &Enviroment) -> Result<super::AgalNumber, crate::runtime::AgalThrow> {
        Ok(super::AgalNumber::new(self.0 as f64))
    }
}
