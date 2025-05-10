use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Sub, SubAssign},
  str::FromStr,
};

const DIVISION_DECIMALS: usize = 100;

#[derive(Debug, Clone, Eq, PartialOrd, Hash)]
pub struct Decimals(Vec<u8>);
impl Decimals {
  pub fn is_zero(&self) -> bool {
    self.0.iter().all(|&x| x == 0)
  }
}

impl Display for Decimals {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut result = String::new();
    for i in 0..self.0.len() {
      let byte = self.0[i];
      let (byte_1, byte_0) = ((byte & 0xF0) >> 4, byte & 0x0F);
      result.push_str(&format!("{:X}{:X}", byte_1, byte_0));
    }
    let clear = result.trim_start_matches('0').to_string();
    if clear.is_empty() {
      return write!(f, "0");
    }
    write!(f, "{}", result.trim_end_matches('0'))
  }
}
impl PartialEq for Decimals {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}
impl Ord for Decimals {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    for i in 0..self.0.len() {
      if self.0[i] != other.0[i] {
        return self.0[i].cmp(&other.0[i]);
      }
    }
    std::cmp::Ordering::Equal
  }
}
impl From<String> for Decimals {
  fn from(value: String) -> Self {
    let mut result = Vec::new();
    let mut i = 0;
    while i < value.len() {
      let char_1 = value.chars().nth(i).unwrap_or('0') as u8 - b'0';
      if char_1 > 9 {
        break;
      }
      let char_0 = value.chars().nth(i + 1).unwrap_or('0') as u8 - b'0';
      if char_0 > 9 {
        break;
      }
      i += 2;
      let byte = (char_1 << 4) | char_0;
      result.push(byte);
    }
    Self(result)
  }
}

#[derive(Debug, Clone, Eq, PartialOrd, Hash)]
pub struct UInt(Vec<u8>);
impl UInt {
  pub fn is_zero(&self) -> bool {
    if self.0.len() == 0 {
      return true;
    }
    self.0.iter().all(|&x| x == 0)
  }
  fn shift_left(&self, n: u8) -> Self {
    let mut result = self.to_string();
    for _ in 0..n {
      result.push('0');
    }
    result.into()
  }
  fn add_digit_mut(&mut self, digit: u8) {
    let mut carry = digit;
    let mut i = self.0.len();
    while carry > 0 && i > 0 {
      i -= 1;
      let lo = self.0[i] & 0x0F;
      let hi = (self.0[i] >> 4) & 0x0F;

      let lo_sum = lo + carry;
      if lo_sum < 10 {
        self.0[i] = (hi << 4) | lo_sum;
        return;
      } else {
        self.0[i] = (hi << 4) | (lo_sum - 10);
        carry = 1;
      }
    }

    if carry > 0 {
      self.0.insert(0, carry); // Insert new digit at front
    }
  }
  fn digit_at(&self, index: usize) -> u8 {
    let byte_index = index / 2;
    let is_high = index % 2 == 0;

    if byte_index >= self.0.len() {
      return 0;
    }

    let byte = self.0[self.0.len() - 1 - byte_index];
    if is_high {
      (byte >> 4) & 0x0F
    } else {
      byte & 0x0F
    }
  }
  fn num_digits(&self) -> usize {
    let len = self.0.len();
    if len == 0 {
      return 0;
    }
    let mut digits = len * 2;
    let last = self.0[0];
    if last & 0x0F == 0x0F {
      digits -= 1;
    }
    digits
  }
  pub fn trim_leading_zeros(mut self) -> Self {
    while self.0.len() > 1 && self.0[0] == 0 {
      self.0.remove(0);
    }
    self
  }
  pub fn digits(&self) -> Vec<u8> {
    self
      .to_string()
      .chars()
      .rev()
      .map(|c| c.to_digit(10).unwrap() as u8)
      .collect()
  }
}

impl Display for UInt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut result = String::new();
    for i in 0..self.0.len() {
      let byte = self.0[i];
      let (byte_1, byte_0) = ((byte & 0xF0) >> 4, byte & 0x0F);
      result.push_str(&format!("{:X}{:X}", byte_1, byte_0));
    }
    let trim_result = result.trim_start_matches('0');
    write!(
      f,
      "{}",
      if trim_result.is_empty() {
        "0"
      } else {
        trim_result
      }
    )
  }
}
impl PartialEq for UInt {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}

impl SubAssign<&UInt> for UInt {
  fn sub_assign(&mut self, rhs: &Self) {
    *self = self.clone() - rhs.clone();
  }
}

impl Add for UInt {
  type Output = Self;
  fn add(self, rhs: Self) -> Self::Output {
    &self + &rhs
  }
}
impl Sub for UInt {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self::Output {
    &self - &rhs
  }
}
impl Mul for UInt {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self::Output {
    &self * &rhs
  }
}
impl Div for UInt {
  type Output = Self;
  fn div(self, rhs: Self) -> Self::Output {
    &self / &rhs
  }
}
impl Add for &UInt {
  type Output = UInt;
  fn add(self, rhs: Self) -> Self::Output {
    let self_vec = self.clone().0;
    let rhs_vec = rhs.clone().0;
    let mut carry = 0;
    let mut result = Vec::new();
    let max_len = self_vec.len().max(rhs_vec.len());
    for i in 0..max_len {
      let i = max_len - 1 - i;
      let a = *self_vec.get(i).unwrap_or(&0);
      let b = *rhs_vec.get(i).unwrap_or(&0);

      let (a_byte_1, a_byte_0) = ((a & 0xF0) >> 4, a & 0x0F);
      let (b_byte_1, b_byte_0) = ((b & 0xF0) >> 4, b & 0x0F);

      let sum_0 = a_byte_0 + b_byte_0 + carry;
      carry = sum_0 / 10;
      let sum_1 = a_byte_1 + b_byte_1 + carry;
      carry = sum_1 / 10;
      let sum = (sum_1 % 10) << 4 | (sum_0 % 10);
      result.push(sum as u8);
    }
    if carry > 0 {
      result.push(carry);
    }
    result.reverse();
    UInt(result)
  }
}
impl Sub for &UInt {
  type Output = UInt;

  fn sub(self, rhs: Self) -> Self::Output {
    let lhs = &self.0;
    let rhs = &rhs.0;

    let mut result = Vec::new();
    let mut carry = 0;

    // Asumimos que lhs >= rhs
    let max_len = lhs.len().max(rhs.len());

    for i in 0..max_len {
      let a = *lhs.get(lhs.len().wrapping_sub(1 + i)).unwrap_or(&0);
      let b = *rhs.get(rhs.len().wrapping_sub(1 + i)).unwrap_or(&0);

      // Extraer nibbles altos y bajos
      let (a1, a0) = ((a >> 4) & 0x0F, a & 0x0F);
      let (b1, b0) = ((b >> 4) & 0x0F, b & 0x0F);

      // Resta del nibble bajo
      let mut sub0 = a0 as i8 - b0 as i8 - carry;
      carry = 0;
      if sub0 < 0 {
        sub0 += 10;
        carry = 1;
      }

      // Resta del nibble alto
      let mut sub1 = a1 as i8 - b1 as i8 - carry;
      carry = 0;
      if sub1 < 0 {
        sub1 += 10;
        carry = 1;
      }

      result.push(((sub1 as u8) << 4) | (sub0 as u8));
    }

    // Eliminar ceros a la izquierda, excepto si es cero solo
    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }

    result.reverse();
    UInt(result)
  }
}
impl Mul for &UInt {
  type Output = UInt;

  fn mul(self, rhs: Self) -> Self::Output {
    let x_digits = self.digits();
    let y_digits = rhs.digits();

    let mut result = vec![0u8; x_digits.len() + y_digits.len()];

    for (i, &x) in x_digits.iter().enumerate() {
      for (j, &y) in y_digits.iter().enumerate() {
        result[i + j] += x * y;
        if result[i + j] >= 10 {
          result[i + j + 1] += result[i + j] / 10;
          result[i + j] %= 10;
        }
      }
    }

    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }

    let result_str: String = result
      .into_iter()
      .rev()
      .map(|d| std::char::from_digit(d as u32, 10).unwrap())
      .collect();
    UInt::from(result_str)
  }
}
impl Div for &UInt {
  type Output = UInt;

  fn div(self, rhs: Self) -> Self::Output {
    if rhs.is_zero() {
      panic!("Division by zero");
    }
    if self < rhs {
      return UInt::from(0); // Si el dividendo es menor que el divisor, el cociente es 0
    }

    let divisor = rhs.clone();
    let mut quotient_digits = Vec::new();

    let mut current = UInt::from(0);

    for i in 0..self.num_digits() {
      current = current.shift_left(1).trim_leading_zeros();
      current.add_digit_mut(self.digit_at(i));

      let mut q_digit = 0;
      for d in (1..=9).rev() {
        let test = &divisor * &UInt::from(d);
        if test <= current {
          q_digit = d;
          break;
        }
      }

      quotient_digits.push(q_digit);
      let sub = &divisor * &UInt::from(q_digit);
      current.sub_assign(&sub);
    }

    // Convert digits to BCD-packed UInt
    let mut result_bytes = Vec::new();
    let mut i = 0;
    while i < quotient_digits.len() {
      let hi = quotient_digits[i];
      let lo = if i + 1 < quotient_digits.len() {
        quotient_digits[i + 1]
      } else {
        0xF // unused nibble
      };
      result_bytes.push((hi << 4) | lo);
      i += 2;
    }

    UInt(result_bytes).trim_leading_zeros()
  }
}
impl Ord for UInt {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let self_len = self.0.len();
    let other_len = other.0.len();
    if self_len != other_len {
      return self_len.cmp(&other_len);
    }
    for i in 0..self_len {
      if self.0[i] != other.0[i] {
        return self.0[i].cmp(&other.0[i]);
      }
    }
    std::cmp::Ordering::Equal
  }
}
impl From<String> for UInt {
  fn from(value: String) -> Self {
    let value = value.chars().rev().collect::<String>();
    let mut result = Vec::new();
    let mut i = 0;
    while i < value.len() {
      let char_0 = value.chars().nth(i).unwrap_or('0') as u8 - b'0';
      if char_0 > 9 {
        break;
      }
      let char_1 = value.chars().nth(i + 1).unwrap_or('0') as u8 - b'0';
      if char_1 > 9 {
        break;
      }
      i += 2;
      let byte = (char_1 << 4) | char_0;
      result.push(byte);
    }
    result.reverse();
    Self(result)
  }
}
impl From<u8> for UInt {
  fn from(value: u8) -> Self {
    if value > 99 {
      panic!("Value must be between 0 and 99");
    }
    let high = value / 10;
    let low = value % 10;
    let byte = (high << 4) | low;
    UInt(vec![byte])
  }
}
#[derive(Debug, Clone, Eq, PartialOrd, Hash)]
pub enum BasicNumber {
  Int(bool, UInt),
  Float(bool, UInt, Decimals),
}
impl BasicNumber {
  pub fn is_zero(&self) -> bool {
    match self {
      Self::Int(_, x) => x.is_zero(),
      Self::Float(_, y_int, y_dec) => y_int.is_zero() && y_dec.to_string() == "0",
    }
  }
  pub fn abs(&self) -> Self {
    match self {
      Self::Int(_, x) => Self::Int(false, x.clone()),
      Self::Float(_, x_int, x_dec) => Self::Float(false, x_int.clone(), x_dec.clone()),
    }
  }
  pub fn floor(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(x_neg.clone(), x.clone()),
      Self::Float(x_neg, x_int, x_dec) => {
        if x_dec.to_string() == "0" {
          return Self::Int(*x_neg, x_int.clone());
        }
        if *x_neg {
          return Self::Int(*x_neg, x_int.clone() - UInt::from(1));
        }
        Self::Int(*x_neg, x_int.clone())
      }
    }
  }
  pub fn ceil(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x, x_dec) => {
        if x_dec.is_zero() {
          return Self::Int(*x_neg, x.clone());
        }
        if *x > UInt::from(0) {
          return Self::Int(*x_neg, x.clone() + UInt::from(1));
        }
        Self::Int(*x_neg, x.clone())
      }
    }
  }
  pub fn round(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x, x_dec) => {
        if x_dec.is_zero() {
          return Self::Int(*x_neg, x.clone());
        }
        for (i, char) in x_dec.to_string().chars().enumerate() {
          let digit = char.to_digit(10).unwrap();
          if digit > 5 || i > 0 {
            // 0.5x | x >= 0
            return Self::Int(*x_neg, x.clone() + 1.into());
          }
          if digit < 5 {
            return Self::Int(*x_neg, x.clone() - 1.into());
          }
        }
        // si el decimal es 0.5 se redondea al par más cercano
        Self::Int(
          *x_neg,
          x.clone()
            + if x.0.last().unwrap_or(&0) % 2 == 0 {
              0
            } else {
              1
            }
            .into(),
        )
      }
    }
  }
  pub fn trunc(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x, _) => Self::Int(*x_neg, x.clone()),
    }
  }
}
impl ToString for BasicNumber {
  fn to_string(&self) -> String {
    match self {
      Self::Int(x_neg, x) => format!("{}{}", if *x_neg { "-" } else { "" }, x),
      Self::Float(x_neg, x, x_dec) => {
        format!("{}{}.{}", if *x_neg { "-" } else { "" }, x, x_dec)
      }
    }
  }
}
impl PartialEq for BasicNumber {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => x_neg == y_neg && x == y,
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        x_neg == y_neg && x == y && x_dec == y_dec
      }
      _ => false,
    }
  }
}

impl Add for BasicNumber {
  type Output = Self;
  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => {
        if x_neg == y_neg {
          return Self::Int(x_neg, x + y);
        }
        if (x.is_zero() && y.is_zero()) || (x == y) {
          return Self::Int(false, UInt::from(0));
        }
        if y > x {
          return Self::Int(y_neg, y - x);
        }
        return Self::Int(x_neg, x - y);
      }
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        if !x_neg && y_neg {
          return Self::Float(false, y, y_dec) - Self::Float(false, x, x_dec);
        } else if x_neg && !y_neg {
          return Self::Float(false, x, x_dec) - Self::Float(false, y, y_dec);
        }
        let mut carry = 0;
        let z_dec = if x_dec.is_zero() && y_dec.is_zero() {
          Decimals(vec![])
        } else if x_dec.is_zero() {
          y_dec.clone()
        } else if y_dec.is_zero() {
          x_dec.clone()
        } else {
          let mut result = Vec::new();
          let max_len = x_dec.0.len().max(y_dec.0.len());
          for i in 0..max_len {
            let i = max_len - 1 - i;
            let a = *x_dec.0.get(i).unwrap_or(&0);
            let b = *y_dec.0.get(i).unwrap_or(&0);
            let (a_byte_1, a_byte_0) = ((a & 0xF0) >> 4, a & 0x0F);
            let (b_byte_1, b_byte_0) = ((b & 0xF0) >> 4, b & 0x0F);

            let sum_0 = a_byte_0 + b_byte_0 + carry;
            carry = sum_0 / 10;
            let sum_1 = a_byte_1 + b_byte_1 + carry;
            carry = sum_1 / 10;
            let sum = (sum_1 % 10) << 4 | (sum_0 % 10);
            result.push(sum as u8);
          }
          result.reverse();
          Decimals(result)
        };
        Self::Float(x_neg, x + y + UInt(vec![carry]), z_dec)
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg, x.clone(), Decimals(vec![])) + Self::Float(y_neg, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg, x.clone(), x_dec.clone()) + Self::Float(y_neg, y, Decimals(vec![]))
      }
    }
  }
}
impl Sub for BasicNumber {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return Self::Int(x_neg, x + y);
        }
        if x == y {
          return Self::Int(false, UInt::from(0));
        }
        if x > y {
          Self::Int(x_neg, x - y)
        } else {
          Self::Int(!y_neg, y - x)
        }
      }

      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return Self::Float(x_neg, x, x_dec) + Self::Float(x_neg, y, y_dec);
        }
        let mut x_dec = x_dec.to_string();
        let mut y_dec = y_dec.to_string();
        let decimals = x_dec.len().max(y_dec.len());
        x_dec.extend(std::iter::repeat('0').take(decimals - x_dec.len()));
        y_dec.extend(std::iter::repeat('0').take(decimals - y_dec.len()));

        let x_full = UInt::from(format!("{}{}", x, x_dec));
        let y_full = UInt::from(format!("{}{}", y, y_dec));
        let (z_neg, z) = match x_full.cmp(&y_full) {
          std::cmp::Ordering::Greater => (x_neg, x_full - y_full),
          std::cmp::Ordering::Less => (!y_neg, y_full - x_full),
          std::cmp::Ordering::Equal => return Self::Int(false, UInt::from(0)),
        };

        let z_str = z.to_string();
        let z_len = z_str.len();

        let (z, z_dec) = if z_len > decimals {
          let (int,dec) = z_str.split_at(z_len - decimals);
          (int.to_string(), dec.to_string())
        } else {
          ("0".to_string(), format!("{:0>width$}", z_str, width = decimals))
        };

        Self::Float(z_neg, UInt::from(z), Decimals::from(z_dec))
      }

      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg, x, Decimals(vec![])) - Self::Float(y_neg, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg, x, x_dec) - Self::Float(y_neg, y, Decimals(vec![]))
      }
    }
  }
}
impl Mul for BasicNumber {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => Self::Int(x_neg ^ y_neg, x * y),
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        let x_dec = x_dec.to_string();
        let y_dec = y_dec.to_string();
        let x_str = format!("{}{}", x, x_dec);
        let x: UInt = x_str.into();
        let y_str = format!("{}{}", y, y_dec);
        let y: UInt = y_str.into();
        let mut decimals = x_dec.len() + y_dec.len();

        let result = x * y;
        let result = result.to_string();

        let int_part: String;
        let mut frac_part = String::new();
        let mut number = result.chars().collect::<Vec<_>>();
        while decimals > 0 {
          decimals -= 1;
          frac_part.push(number.pop().unwrap_or('0'));
        }
        int_part = if number.is_empty() {
          "0".to_string()
        } else {
          number.iter().collect()
        };

        let frac_part = frac_part.chars().rev().collect::<String>();

        Self::Float(x_neg ^ y_neg, int_part.into(), frac_part.into())
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), Decimals(vec![])) * Self::Float(false, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), x_dec.clone())
          * Self::Float(false, y, Decimals(vec![]))
      }
    }
  }
}
impl Div for BasicNumber {
  type Output = Self;
  fn div(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => Self::Int(x_neg ^ y_neg, x / y),
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        let mut x_dec = x_dec.to_string().trim_end_matches('0').to_string();
        let mut y_dec = y_dec.to_string().trim_end_matches('0').to_string();
        let max_dec = x_dec.len().max(y_dec.len());
        x_dec.extend(std::iter::repeat('0').take(max_dec + DIVISION_DECIMALS - x_dec.len()));
        y_dec.extend(std::iter::repeat('0').take(max_dec - y_dec.len()));
        let x_str = format!("{}{}", x, x_dec);
        let y_str = format!("{}{}", y, y_dec);
        let mut decimals = max_dec + DIVISION_DECIMALS;

        let result = UInt::from(x_str) / UInt::from(y_str);
        let result = result.to_string();

        let int_part: String;
        let mut frac_part = String::new();
        let mut number = result.chars().collect::<Vec<_>>();
        while decimals > 0 {
          decimals -= 1;
          frac_part.push(number.pop().unwrap_or('0'));
        }
        int_part = if number.is_empty() {
          "0".to_string()
        } else {
          number.iter().collect()
        };

        let frac_part = frac_part.chars().rev().collect::<String>();

        Self::Float(x_neg ^ y_neg, int_part.into(), frac_part.into())
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), Decimals(vec![])) / Self::Float(false, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), x_dec.clone())
          / Self::Float(false, y, Decimals(vec![]))
      }
    }
  }
}
impl Neg for BasicNumber {
  type Output = Self;
  fn neg(self) -> Self::Output {
    match self {
      Self::Int(x_neg, x) => Self::Int(!x_neg, x),
      Self::Float(x_neg, x, x_dec) => Self::Float(!x_neg, x, x_dec),
    }
  }
}
impl Ord for BasicNumber {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match (self, other) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => {
        if x.is_zero() && y.is_zero() {
          return std::cmp::Ordering::Equal;
        }

        if *x_neg && !*y_neg {
          return std::cmp::Ordering::Greater;
        } else if !*x_neg && *y_neg {
          return std::cmp::Ordering::Less;
        }

        if *x_neg && *y_neg {
          return x.cmp(&y).reverse();
        }
        x.cmp(&y)
      }
      (Self::Float(x_neg, x_int, x_dec), Self::Float(y_neg, y_int, y_dec)) => {
        if x_int.is_zero() && y_int.is_zero() {
          return x_dec.cmp(y_dec);
        }
        if *x_neg && !*y_neg {
          return std::cmp::Ordering::Greater;
        } else if !*x_neg && *y_neg {
          return std::cmp::Ordering::Less;
        }
        if x_int == y_int {
          if *x_neg && *y_neg {
            return y_dec.cmp(x_dec).reverse();
          }
          return x_dec.cmp(y_dec);
        }
        return x_int.cmp(y_int);
      }
      (Self::Int(x_neg, x_int), y) => Self::Float(*x_neg, x_int.clone(), Decimals(vec![0])).cmp(y),
      (x, Self::Int(y_neg, y_int)) => Self::Float(*y_neg, y_int.clone(), Decimals(vec![0]))
        .cmp(x)
        .reverse(),
    }
  }
}
impl From<String> for BasicNumber {
  fn from(value: String) -> Self {
    let split = value.split('.');
    let int_part = split.clone().nth(0).unwrap_or("0").to_string();
    let frac_part = split.clone().nth(1).unwrap_or("0").to_string();
    Self::Float(false, int_part.into(), frac_part.into())
  }
}
#[derive(Debug, Clone, Eq, Default, Hash)]
pub enum Number {
  #[default]
  NaN,
  Infinity,
  NegativeInfinity,
  Basic(BasicNumber),
  Complex(BasicNumber, BasicNumber),
}
impl Number {
  pub fn ceil(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegativeInfinity => Self::NegativeInfinity,
      Self::Basic(x) => Self::Basic(x.ceil()),
      Self::Complex(x, y) => {
        let x = x.ceil();
        let y = y.ceil();
        if y.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x, y)
      }
    }
  }
  pub fn floor(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegativeInfinity => Self::NegativeInfinity,
      Self::Basic(x) => Self::Basic(x.floor()),
      Self::Complex(x, y) => {
        let x = x.floor();
        let y = y.floor();
        if y.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x, y)
      }
    }
  }
  pub fn trunc(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegativeInfinity => Self::NegativeInfinity,
      Self::Basic(x) => Self::Basic(x.trunc()),
      Self::Complex(x, y) => {
        let x = x.trunc();
        let y = y.trunc();
        if y.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x, y)
      }
    }
  }
  pub fn round(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegativeInfinity => Self::NegativeInfinity,
      Self::Basic(x) => Self::Basic(x.round()),
      Self::Complex(x, y) => {
        let x = x.round();
        let y = y.round();
        if y.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x, y)
      }
    }
  }
  pub const fn is_nan(&self) -> bool {
    matches!(self, Self::NaN)
  }
  pub const fn is_infinite(&self) -> bool {
    matches!(self, Self::Infinity | Self::NegativeInfinity)
  }
  pub fn is_zero(&self) -> bool {
    match self {
      Self::NaN | Self::Infinity | Self::NegativeInfinity => false,
      Self::Basic(x) => x.is_zero(),
      Self::Complex(x, y) => x.is_zero() && y.is_zero(),
    }
  }
  pub fn abs(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::Infinity,
      Self::NegativeInfinity => Self::Infinity,
      Self::Basic(x) => Self::Basic(x.abs()),
      Self::Complex(x, y) => Self::Complex(x.abs(), y.abs()),
    }
  }
  pub fn from_str_radix(value: &str, radix: u8) -> Self {
    if radix < 2 || radix > 36 {
      return Self::NaN;
    }
    i32::from_str_radix(value, radix as u32)
      .map(|v| v.to_string().parse::<Self>().unwrap_or_default())
      .unwrap_or_default()
  }
}
impl From<&char> for Number {
  fn from(value: &char) -> Self {
    Self::Basic(BasicNumber::Int(false, value.to_string().into()))
  }
}
impl FromStr for Number {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "infinito" {
      return Ok(Self::Infinity);
    }
    if s == "-infinito" {
      return Ok(Self::NegativeInfinity);
    }
    if s == "NeN" {
      return Ok(Self::NaN);
    }
    if s.contains("i") {
      let parts: Vec<&str> = s.split("i").collect();
      let i_exp = parts.len() - 1;
      let number = parts.join("").into();
      for (i, part) in parts.iter().enumerate() {
        if i == 0 {
          continue;
        }
        if part
          .chars()
          .any(|c| c.is_digit(10) || c == '-' || c == '+' || c == '.')
        {
          return Err(format!(
            "No se puede poner numeros despues de las constantes: {}",
            part
          ));
        }
      }
      let i_exp = i_exp % 4;

      return if i_exp == 1 {
        Ok(Self::Complex(
          BasicNumber::Int(false, UInt::from(0)),
          number,
        ))
      } else if i_exp == 2 {
        Ok(Self::Basic(-number))
      } else if i_exp == 3 {
        Ok(Self::Complex(
          BasicNumber::Int(false, UInt::from(0)),
          -number,
        ))
      } else {
        Ok(Self::Basic(number))
      };
    }
    if s.contains(".") {
      let parts: Vec<&str> = s.split(".").collect();
      let int_part = parts[0].to_string();
      let frac_part = parts[1].to_string();
      return Ok(Self::Basic(BasicNumber::Float(
        false,
        int_part.into(),
        frac_part.into(),
      )));
    }
    if s.chars().all(|c| c.is_digit(10)) {
      return Ok(Self::Basic(BasicNumber::Int(false, s.to_string().into())));
    }
    Err(format!("No se puede convertir el string '{s}' a un número",))
  }
}
impl From<usize> for Number {
  fn from(value: usize) -> Self {
    Self::Basic(BasicNumber::Int(false, value.to_string().into()))
  }
}
impl ToString for Number {
  fn to_string(&self) -> String {
    match self {
      Self::NaN => "NeN".to_string(),
      Self::Infinity => "infinito".to_string(),
      Self::NegativeInfinity => "-infinito".to_string(),
      Self::Basic(x) => x.to_string(),
      Self::Complex(x, y) => {
        if x.is_zero() && y.is_zero() {
          return "0".to_string();
        }
        if x.is_zero() {
          return format!("{}i", y.to_string());
        }
        if y.is_zero() {
          return x.to_string();
        }
        return format!("{} + {}i", x.to_string(), y.to_string());
      }
    }
  }
}
impl PartialEq for Number {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}

impl Add for Number {
  type Output = Self;
  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::NaN, _) => Self::NaN,
      (_, Self::NaN) => Self::NaN,
      (Self::Infinity, _) => Self::Infinity,
      (_, Self::Infinity) => Self::Infinity,
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::NegativeInfinity,
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(x + y),
      (Self::Complex(x, y), Self::Complex(a, b)) => Self::Complex(x + a, y + b),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x + a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(a + x, b)
      }
    }
  }
}
impl Sub for Number {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::NaN, _) => Self::NaN,
      (_, Self::NaN) => Self::NaN,
      (_, Self::Infinity) => Self::NegativeInfinity,
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (Self::Infinity, _) => Self::Infinity,
      (_, Self::NegativeInfinity) => Self::Infinity,
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(x - y),
      (Self::Complex(x, y), Self::Complex(a, b)) => Self::Complex(x - a, y - b),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x - a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(a - x, b)
      }
    }
  }
}
impl Mul for Number {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::NaN, _) => Self::NaN,
      (_, Self::NaN) => Self::NaN,
      (Self::Infinity, _) => Self::Infinity,
      (_, Self::Infinity) => Self::Infinity,
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::NegativeInfinity,
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(x * y),
      (Self::Complex(a, b), Self::Complex(c, d)) => Self::Complex(
        (a.clone() * c.clone()) - (b.clone() * d.clone()),
        (a * d) + (c * b),
      ),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x.clone() * a, x * b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(a * x.clone(), b * x)
      }
    }
  }
}
impl Div for Number {
  type Output = Self;
  fn div(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::NaN, _) => Self::NaN,
      (_, Self::NaN) => Self::NaN,
      (Self::Infinity, _) => Self::Infinity,
      (_, Self::Infinity) => Self::Basic(BasicNumber::Int(false, UInt::from(0))),
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::Basic(BasicNumber::Int(false, UInt::from(0))),
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(x / y),
      (Self::Complex(a, b), Self::Complex(c, d)) => {
        let conj = (c.clone() * c.clone()) + (d.clone() * d.clone());

        Self::Complex(
          ((a.clone() * c.clone()) + (b.clone() * d.clone())) / conj.clone(),
          ((b * c) - (a * d)) / conj,
        )
      }
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(x / a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(a / x, b)
      }
    }
  }
}
impl Neg for Number {
  type Output = Self;
  fn neg(self) -> Self::Output {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::NegativeInfinity,
      Self::NegativeInfinity => Self::Infinity,
      Self::Basic(x) => Self::Basic(-x),
      Self::Complex(x, y) => Self::Complex(-x, -y),
    }
  }
}
impl PartialOrd for Number {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (Self::NaN, _) => None,
      (_, Self::NaN) => None,
      (Self::Infinity, _) => Some(std::cmp::Ordering::Greater),
      (_, Self::Infinity) => Some(std::cmp::Ordering::Less),
      (Self::NegativeInfinity, _) => Some(std::cmp::Ordering::Less),
      (_, Self::NegativeInfinity) => Some(std::cmp::Ordering::Greater),
      (Self::Basic(x), Self::Basic(y)) => x.partial_cmp(y),
      (Self::Complex(a, b), Self::Complex(c, d)) => a.partial_cmp(c).or_else(|| b.partial_cmp(d)),
      (Self::Basic(x), Self::Complex(a, b)) => x.partial_cmp(a).or_else(|| x.partial_cmp(b)),
      (Self::Complex(a, b), Self::Basic(x)) => a.partial_cmp(x).or_else(|| b.partial_cmp(x)),
    }
  }
}
