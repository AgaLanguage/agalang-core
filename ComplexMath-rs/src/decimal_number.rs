use crate::big_int;
use crate::ubig_int;
pub struct DecimalNumber {
    integer: big_int::BigInt,
    decimal: ubig_int::UBigInt,
}

impl DecimalNumber {
    pub fn new(s: &str) -> DecimalNumber {
        if !s.contains('.') {
            return DecimalNumber {
                integer: big_int::BigInt::new(s),
                decimal: ubig_int::UBigInt::new("0"),
            };
        }
        let parts: Vec<&str> = s.split('.').collect();
        let integer = big_int::BigInt::new(parts[0]);
        let decimal = ubig_int::UBigInt::new(parts[1].chars().rev().collect::<String>().as_str());
        DecimalNumber { integer, decimal }
    }
    fn decimal_value(&self) -> String {
        self.decimal.to_string().chars().rev().collect::<String>()
    }
    pub fn to_string(&self) -> String {
        let mut s = self.integer.to_string();
        s.push('.');
        s.push_str(&self.decimal_value());
        s
    }
    pub fn to_big_int(&self, decimals: usize) -> big_int::BigInt {
        let int = self.integer.to_string();
        let vec_dec = self.decimal_value().chars().collect::<Vec<char>>();
        let mut dec = "".to_string();
        for i in 0..decimals {
            let c: char = if i < vec_dec.len() { vec_dec[i] } else { '0' };
            dec.push(c);
        }
        big_int::BigInt::new(format!("{}{}", int, dec).as_str())
    }
    pub fn add(&self, other: &DecimalNumber) -> DecimalNumber {
        let max_decimals = self.decimal.len().max(other.decimal.len());
        let a = self.to_big_int(max_decimals);
        let b = other.to_big_int(max_decimals);
        let result = a.add(&b);
        let result_str = result.to_string();
        let int = &result_str[0..result_str.len() - max_decimals];
        let dec = &result_str[result_str.len() - max_decimals..];
        DecimalNumber::new(format!("{}.{}", int, dec).as_str())
    }
    pub fn substract(&self, other: &DecimalNumber) -> DecimalNumber {
        let max_decimals = self.decimal.len().max(other.decimal.len());
        let a = self.to_big_int(max_decimals);
        let b = other.to_big_int(max_decimals);
        let result = a.substract(&b);
        let result_str = result.to_string();
        let int = &result_str[0..result_str.len() - max_decimals];
        let dec = &result_str[result_str.len() - max_decimals..];
        DecimalNumber::new(format!("{}.{}", int, dec).as_str())
    }
    pub fn multiply(&self, other: &DecimalNumber) -> DecimalNumber {
        let a = self.to_big_int(self.decimal.len());
        let b = other.to_big_int(other.decimal.len());
        let result = a.multiply(&b);
        let result_str = result.to_string();
        let decimal_len = self.decimal.len() + other.decimal.len();
        let int = &result_str[0..result_str.len() - decimal_len];
        let dec = &result_str[result_str.len() - decimal_len..];
        DecimalNumber::new(format!("{}.{}", int, dec).as_str())
    }
    pub fn divide(&self, other: &DecimalNumber) -> DecimalNumber {
        let a = self.to_big_int(self.decimal.len());
        let b = other.to_big_int(other.decimal.len());
        let result = a.divide(&b);
        let result_str = result.to_string();
        let decimal_len = self.decimal.len() + other.decimal.len();
        let int = &result_str[0..result_str.len() - decimal_len];
        let dec = &result_str[result_str.len() - decimal_len..];
        DecimalNumber::new(format!("{}.{}", int, dec).as_str())
    }
    pub fn equals(&self, other: &DecimalNumber) -> bool {
        self.integer.equals(&other.integer) && self.decimal.equals(&other.decimal)
    }
}

impl std::fmt::Display for DecimalNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}