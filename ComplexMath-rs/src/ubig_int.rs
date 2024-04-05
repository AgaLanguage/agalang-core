pub struct UBigInt {
    digits: Vec<u8>,
}

impl UBigInt {
    pub fn s_use(a: &UBigInt, b: &UBigInt, cb: fn(&str, &str) -> String) -> UBigInt {
        let a_string = a.to_string();
        let b_string = b.to_string();
        let a_str = a_string.as_str();
        let b_str = b_string.as_str();
        let result = cb(a_str, b_str);
        UBigInt::new(result.as_str())
    }
    pub fn s_use_bool(a: &UBigInt, b: &UBigInt, cb: fn(&str, &str) -> bool) -> bool {
        let a_string = a.to_string();
        let b_string = b.to_string();
        let a_str = a_string.as_str();
        let b_str = b_string.as_str();
        cb(a_str, b_str)
    }
    // 000123 -> 123
    pub fn s_clean(s: &str) -> &str {
        let clean_string = s.trim_start_matches('0');
        if clean_string.is_empty() {
            return "0";
        }
        clean_string
    }
    // x == y
    pub fn s_equals(a: &str, b: &str) -> bool {
        UBigInt::s_clean(a) == UBigInt::s_clean(b)
    }
    // x < y
    pub fn s_less_than(a: &str, b: &str) -> bool {
        if UBigInt::s_equals(a, b) {
            return false;
        }
        let a_len = a.len();
        let b_len = b.len();
        if a_len != b_len {
            return a_len < b_len;
        }
        for (i, c) in a.chars().enumerate() {
            let a_char = c.to_ascii_lowercase();
            let b_char = b.chars().nth(i).unwrap().to_ascii_lowercase();
            if a_char != b_char {
                return a_char < b_char;
            }
        }
        false
    }
    // x == 0
    pub fn s_is_zero(s: &str) -> bool {
        UBigInt::s_equals(s, "0")
    }
    // x + y
    pub fn s_add(a: &str, b: &str) -> String {
        if UBigInt::s_is_zero(a) {
            return b.to_string();
        }
        if UBigInt::s_is_zero(b) {
            return a.to_string();
        }
        let a_len = a.len();
        let b_len = b.len();
        let max_size = if a_len > b_len { a_len } else { b_len };
        let mut result = String::new();
        let mut carry: u8 = 0;
        for i in 1..=max_size {
            let a_ind = if a_len >= i { a_len - i } else { 0 };
            let b_ind = if b_len >= i { b_len - i } else { 0 };
            // z_len >= i == x_ind >= 0, no valid overflow
            let x = if a_len >= i {
                a.chars().nth(a_ind).unwrap()
            } else {
                '0'
            };
            let y = if b_len >= i {
                b.chars().nth(b_ind).unwrap()
            } else {
                '0'
            };
            let x_digit = x.to_digit(10).unwrap() as u8;
            let y_digit = y.to_digit(10).unwrap() as u8;
            let sum = x_digit + y_digit + carry;
            carry = sum / 10;
            result = format!("{}{}", sum % 10, result);
        }
        result = format!("{}{}", carry, result);
        UBigInt::s_clean(result.as_str()).to_string()
    }
    // |x - y|
    pub fn s_substract(a: &str, b: &str) -> String {
        if UBigInt::s_equals(a, b) {
            return "0".to_string();
        }
        if UBigInt::s_is_zero(a) {
            return b.to_string();
        }
        if UBigInt::s_is_zero(b) {
            return a.to_string();
        }
        let mut x = a.to_string();
        let mut y = b.to_string();
        if UBigInt::s_less_than(x.as_str(), y.as_str()) {
            x = b.to_string();
            y = a.to_string();
        }
        let x_len = x.len();
        let y_len = y.len();
        let max_size = if x_len > y_len { x_len } else { y_len };
        let mut result = String::new();
        let mut carry: i8 = 0;
        for i in 1..=max_size {
            let x_ind = if x_len >= i { x_len - i } else { 0 };
            let y_ind = if y_len >= i { y_len - i } else { 0 };
            let x = if x_len >= i {
                x.chars().nth(x_ind).unwrap()
            } else {
                '0'
            };
            let y = if y_len >= i {
                y.chars().nth(y_ind).unwrap()
            } else {
                '0'
            };
            let x_digit = x.to_digit(10).unwrap() as i8;
            let y_digit = y.to_digit(10).unwrap() as i8;
            let sub = x_digit - y_digit - carry;
            carry = if sub < 0 { 1 } else { 0 };
            result = format!("{}{}", if sub < 0 { sub + 10 } else { sub }, result);
        }
        result = format!("{}{}", carry, result);
        UBigInt::s_clean(result.as_str()).to_string()
    }
    // x * y
    pub fn s_multiply(a: &str, b: &str) -> String {
        if UBigInt::s_is_zero(a) || UBigInt::s_is_zero(b) {
            return "0".to_string();
        }
        let a_len = a.len();
        let b_len = b.len();
        let mut result = vec![0; a_len + b_len];
        for (i, c) in a.chars().rev().enumerate() {
            let a_digit = c.to_digit(10).unwrap() as u8;
            for (j, d) in b.chars().rev().enumerate() {
                let b_digit = d.to_digit(10).unwrap() as u8;
                let product = a_digit * b_digit;
                let sum = result[i + j] + product;
                result[i + j] = sum % 10;
                result[i + j + 1] += sum / 10;
            }
        }
        let mut result_reverse_string = String::new();
        for d in result.iter().rev() {
            result_reverse_string = format!("{}{}", d, result_reverse_string);
        }
        let result_string = result_reverse_string.chars().rev().collect::<String>();
        UBigInt::s_clean(result_string.as_str()).to_string()
    }
    // (x / y, x % y)
    pub fn s_divide(a: &str, b: &str) -> (String, String) {
        if UBigInt::s_is_zero(b) {
            panic!("Division by zero");
        }
        if UBigInt::s_less_than(a, b) {
            return ("0".to_string(), a.to_string());
        }
        if UBigInt::s_equals(a, b) {
            return ("1".to_string(), "0".to_string());
        }
        if UBigInt::s_equals(b, "1") {
            return (a.to_string(), "0".to_string());
        }
        let mut x = a.to_string();
        let y = b.to_string();
        let mut count = "0".to_string();
        while !UBigInt::s_less_than(x.as_str(), y.as_str()) {
            let x_len = x.len();
            let y_len = y.len();
            let len_diff = x_len - y_len;
            if x_len < y_len {
                break;
            }

            let aux = format!("{}{}", y, "0".repeat(len_diff));
            let increment = format!("1{}", "0".repeat(len_diff));
            if UBigInt::s_less_than(x.as_str(), aux.as_str()) && len_diff > 0 {
                let aux = format!("{}{}", y, "0".repeat(len_diff - 1));
                let increment = format!("1{}", "0".repeat(len_diff - 1));
                x = UBigInt::s_substract(x.as_str(), aux.as_str());
                count = UBigInt::s_add(count.as_str(), increment.as_str());
            } else {
                x = UBigInt::s_substract(x.as_str(), aux.as_str());
                count = UBigInt::s_add(count.as_str(), increment.as_str());
            }
        }
        (count, x)
    }
    pub fn new(mut s: &str) -> UBigInt {
        let mut digits = vec![];
        let mut index = 0;
        s = UBigInt::s_clean(s);
        if s.is_empty() {
            digits.push(0);
            return UBigInt { digits: digits };
        }
        for c in s.chars().rev() {
            let digit = c.to_digit(10).unwrap() as u8;
            if index % 2 == 0 {
                digits.push(digit);
            } else {
                digits[index / 2] += digit << 4;
            }
            index += 1;
        }
        let digits_end = digits.to_vec();
        UBigInt { digits: digits_end }
    }
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for d in &self.digits {
            s = format!("{:02x}{}", d, s);
        }
        s = s.trim_start_matches('0').to_string();
        if s.is_empty() {
            s.push('0');
        }
        s
    }
    pub fn equals(&self, other: &UBigInt) -> bool {
        UBigInt::s_use_bool(self, other, UBigInt::s_equals)
    }
    pub fn less_than(&self, other: &UBigInt) -> bool {
        UBigInt::s_use_bool(self, other, UBigInt::s_less_than)
    }
    pub fn add(&self, other: &UBigInt) -> UBigInt {
        UBigInt::s_use(self, other, UBigInt::s_add)
    }
    pub fn substract(&self, other: &UBigInt) -> UBigInt {
        UBigInt::s_use(self, other, UBigInt::s_substract)
    }
    pub fn multiply(&self, other: &UBigInt) -> UBigInt {
        UBigInt::s_use(self, other, UBigInt::s_multiply)
    }
    pub fn div_mod(&self, other: &UBigInt) -> (UBigInt, UBigInt) {
        let a_string = self.to_string();
        let b_string = other.to_string();
        let a_str = a_string.as_str();
        let b_str = b_string.as_str();
        let result = UBigInt::s_divide(a_str, b_str);
        (
            UBigInt::new(result.0.as_str()),
            UBigInt::new(result.1.as_str()),
        )
    }
    pub fn divide(&self, other: &UBigInt) -> UBigInt {
        self.div_mod(other).0
    }
    pub fn modulo(&self, other: &UBigInt) -> UBigInt {
        self.div_mod(other).1
    }
    pub fn len(&self) -> usize {
        self.to_string().len()
    }
}

impl std::fmt::Display for UBigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl Clone for UBigInt {
    fn clone(&self) -> UBigInt {
        UBigInt::new(self.to_string().as_str())
    }
}