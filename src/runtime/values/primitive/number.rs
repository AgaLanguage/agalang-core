use crate::runtime::{
    delete_property_error, env::RefEnviroment, get_instance_property_error, get_property_error,
    set_property_error, AgalArray, AgalThrow, AgalValuable, AgalValue, RefAgalValue, Stack,
};

use super::{AgalBoolean, AgalString};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct AgalNumber(f64);
impl AgalNumber {
    pub fn new(value: f64) -> AgalNumber {
        AgalNumber(value)
    }
    pub fn to_f64(&self) -> f64 {
        self.0
    }
    pub fn multiply(&self, other: AgalNumber) -> AgalNumber {
        AgalNumber::new(self.0 * other.0)
    }
}
impl AgalValuable for AgalNumber {
    fn to_value(self) -> AgalValue {
        AgalValue::Number(self)
    }
    fn binary_operation(
        &self,
        stack: &Stack,
        _env: RefEnviroment,
        operator: String,
        other: RefAgalValue,
    ) -> RefAgalValue {
        let other: &AgalValue = &other.borrow();
        match other {
            AgalValue::Number(other) => {
                let number = match operator.as_str() {
                    "+" => AgalNumber::new(self.0 + other.0),
                    "-" => AgalNumber::new(self.0 - other.0),
                    "*" => AgalNumber::new(self.0 * other.0),
                    "/" => {
                        if other.0 == 0f64 {
                            return AgalValue::Throw(AgalThrow::Params {
                                type_error: crate::internal::ErrorNames::TypeError,
                                message: "No se puede dividir por cero".to_string(),
                                stack: Box::new(stack.clone()),
                            })
                            .as_ref();
                        }
                        AgalNumber::new(self.0 / other.0)
                    }
                    "%" => {
                        if other.0 == 0f64 {
                            return AgalValue::Throw(AgalThrow::Params {
                                type_error: crate::internal::ErrorNames::TypeError,
                                message: "No se puede dividir por cero".to_string(),
                                stack: Box::new(stack.clone()),
                            })
                            .as_ref();
                        }
                        AgalNumber::new(self.0 % other.0)
                    }
                    "==" => {
                        return AgalValue::Boolean(AgalBoolean::new(self.0 == other.0)).as_ref()
                    }
                    "!=" => {
                        return AgalValue::Boolean(AgalBoolean::new(self.0 != other.0)).as_ref()
                    }
                    "<" => return AgalValue::Boolean(AgalBoolean::new(self.0 < other.0)).as_ref(),
                    "<=" => {
                        return AgalValue::Boolean(AgalBoolean::new(self.0 <= other.0)).as_ref()
                    }
                    ">" => return AgalValue::Boolean(AgalBoolean::new(self.0 > other.0)).as_ref(),
                    ">=" => {
                        return AgalValue::Boolean(AgalBoolean::new(self.0 >= other.0)).as_ref()
                    }
                    "&&" => {
                        return (if self.0 == 0f64 { self } else { other })
                            .to_value()
                            .as_ref()
                    }
                    "||" => {
                        return (if self.0 != 0f64 { self } else { other })
                            .to_value()
                            .as_ref()
                    }
                    _ => {
                        return AgalValue::Throw(AgalThrow::Params {
                            type_error: crate::internal::ErrorNames::TypeError,
                            message: format!("Operador {} no soportado", operator),
                            stack: Box::new(stack.clone()),
                        })
                        .as_ref();
                    }
                };
                AgalValue::Number(number).as_ref()
            }
            _ => AgalValue::Throw(AgalThrow::Params {
                type_error: crate::internal::ErrorNames::TypeError,
                message: "No se puede operar con un valor no numerico".to_string(),
                stack: Box::new(stack.clone()),
            })
            .as_ref(),
        }
    }
    fn to_agal_number(self, _: &Stack, _: RefEnviroment) -> Result<AgalNumber, AgalThrow> {
        Ok(self)
    }
    fn to_agal_boolean(self, _: &Stack, _: RefEnviroment) -> Result<AgalBoolean, AgalThrow> {
        Ok(AgalBoolean(self.0 != 0f64))
    }
    fn to_agal_string(self, _: &Stack, _: RefEnviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(self.0.to_string()))
    }
    fn to_agal_console(self, _: &Stack, _: RefEnviroment) -> Result<AgalString, AgalThrow> {
        Ok(AgalString::from_string(format!("\x1b[33{}\x1b[39", self.0)))
    }
    fn get_instance_property(self, stack: &Stack, env: RefEnviroment, key: String) -> RefAgalValue {
        let value = AgalValue::Number(self);
        get_instance_property_error(stack, env, key, value)
    }
    fn call(
        self,
        stack: &Stack,
        env: RefEnviroment,
        _: RefAgalValue,
        list: Vec<RefAgalValue>,
    ) -> RefAgalValue {
        let value = list.get(0);
        if value.is_none() {
            return AgalValue::Number(self).as_ref();
        }
        let value = value.unwrap();
        let other = value.borrow().clone().to_agal_number(stack, env);
        if other.is_err() {
            return other.err().unwrap().to_value().as_ref();
        }
        let other = other.ok().unwrap();
        let number = self.multiply(other);
        AgalValue::Number(number).as_ref()
    }

    fn get_keys(self) -> Vec<String> {
        std::vec![]
    }

    fn get_length(self) -> usize {
        0
    }

    fn to_agal_array(self, stack: &Stack) -> Result<AgalArray, AgalThrow> {
        Err(AgalThrow::Params {
            type_error: crate::internal::ErrorNames::CustomError("Error Iterable".to_string()),
            message: "El valor no es iterable".to_string(),
            stack: Box::new(stack.clone()),
        })
    }

    fn to_agal_value(self, stack: &Stack, env: RefEnviroment) -> Result<AgalString, AgalThrow> {
        self.to_agal_console(stack, env)
    }

    fn get_object_property(self, stack: &Stack, env: RefEnviroment, key: String) -> RefAgalValue {
        get_property_error(stack, env, key)
    }

    fn set_object_property(
        mut self,
        stack: &Stack,
        env: RefEnviroment,
        key: String,
        _value: RefAgalValue,
    ) -> RefAgalValue {
        set_property_error(stack, env, key, "No se puede asignar".to_string())
    }

    fn delete_object_property(mut self, stack: &Stack, env: RefEnviroment, key: String) {
        delete_property_error(stack, env, key);
    }

    fn construct(self, _: &Stack, _: RefEnviroment, _: Vec<RefAgalValue>) -> RefAgalValue {
        AgalValue::Never.as_ref()
    }
}
/* 
#[derive(Clone)]
pub struct Number(Vec<u8>);

impl Number {
    pub fn from_str10(string: &str) -> Self {
        let bcd_list = {
            let string: String = {
                let len = string.len();
                if (len % 2) == 1 {
                    format!("0{}", string)
                } else {
                    string.to_string()
                }
            };
            let char_list: Vec<char> = string.chars().collect();
            let bytes_list = char_list.chunks(2).map(|c| {
                let first = c[0].to_digit(10);
                if first.is_none() {
                    panic!("'{}' not is a valid decimal number", c[1]);
                }
                let first = first.unwrap() as u8;
                let second = c[1].to_digit(10);
                if second.is_none() {
                    panic!("'{}' not is a valid decimal number", c[1]);
                }
                let second = second.unwrap() as u8;
                let mut byte = 0u8;
                byte += second;
                byte += first << 4;
                byte
            });
            let list: Vec<u8> = bytes_list.collect();
            list
        };
        Self::from_bcd_list(bcd_list)
    }
    pub fn from_bcd_list(bcd_list: Vec<u8>) -> Self {
        let mut bin_list: Vec<u8> = Vec::new();
        let len = bcd_list.len();
        let max = len - 1;
        for x in 0..len {
            let x = max - x;
            let value = bcd_list[x];
            let first = value >> 4;
            let second = (value << 4) >> 4;

            let first = first * 10;
            if x == max {
                let value = first + second;
                bin_list.push(value);
                continue;
            }
            let second = second * 10;
            let prev_bin = bin_list[0];
            let prev = (prev_bin >> 4) + second;
            let prev = (prev_bin & 0xFF) + (prev << 4);
            let second = second >> 4;
            let value = first + second;
            bin_list[0] = prev;
            bin_list.insert(0, value);
        }
        Self(bin_list)
    }
}

fn Vecu8toBin(vec: &Vec<u8>) -> String {
    let mut data = String::new();
    for x in vec {
        let string = format!("{:08b} ", x);
        data.push_str(&string);
    }
    data
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = Vecu8toBin(&self.0);
        write!(f, "{data}")
    }
}
*/