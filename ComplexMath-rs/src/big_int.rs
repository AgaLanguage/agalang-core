use crate::ubig_int;
pub struct BigInt {
    value: ubig_int::UBigInt,
    sign: bool,
}
impl BigInt {
    pub fn new(s: &str) -> BigInt {
        let sign = s.chars().nth(0).unwrap() == '-';
        let value = ubig_int::UBigInt::new(s);
        BigInt { value, sign }
    }
    pub fn to_string(&self) -> String {
        let mut s = self.value.to_string();
        if self.sign {
            s.insert(0, '-');
        }
        s
    }
    pub fn add(&self, other: &BigInt) -> BigInt {
        if self.sign == other.sign {
            let value = self.value.add(&other.value).clone();
            return BigInt {
                value,
                sign: self.sign,
            };
        }
        let value = self.value.substract(&other.value).clone();
        let sign = if self.value.less_than(&other.value) {
            other.sign
        } else {
            self.sign
        };
        return BigInt { value, sign };
    }
    pub fn substract(&self, other: &BigInt) -> BigInt {
        if self.sign == other.sign {
            let value = self.value.substract(&other.value).clone();
            let sign = self.value.less_than(&other.value);
            return BigInt { value, sign };
        }
        let value = self.value.add(&other.value).clone();
        let sign = self.sign;
        BigInt { value, sign }
    }
    pub fn multiply(&self, other: &BigInt) -> BigInt {
        let value = self.value.multiply(&other.value);
        let sign = self.sign != other.sign;
        BigInt { value, sign }
    }
    pub fn div_mod(&self, other: &BigInt) -> (BigInt, BigInt) {
      let result = self.value.div_mod(&other.value);
      let divide_result = BigInt { value: result.0, sign: self.sign != other.sign };
      let mod_result = BigInt { value: result.1, sign: self.sign };
      (divide_result, mod_result)
    }
    pub fn divide(&self, other: &BigInt) -> BigInt {
        self.div_mod(other).0
    }
    pub fn modulo(&self, other: &BigInt) -> BigInt {
        self.div_mod(other).1
    }
    pub fn equals(&self, other: &BigInt) -> bool {
        self.sign == other.sign && self.value.equals(&other.value)
    }
    pub fn len(&self) -> usize {
        self.value.len()
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
