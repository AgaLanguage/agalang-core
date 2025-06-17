use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub, SubAssign},
  str::FromStr,
};

use crate::{
  util::{OnError, OnSome},
  Encode, StructTag,
};

const DIVISION_DECIMALS: usize = 100;

#[derive(Clone, Eq, Hash, Debug)]
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
    let clear = result.trim_end_matches('0').to_string();
    if clear.is_empty() {
      return write!(f, "0");
    }
    write!(f, "{}", clear)
  }
}
impl PartialEq for Decimals {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}
impl PartialOrd for Decimals {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.cmp(other).into()
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

#[derive(Clone, Eq, Hash, Debug)]
pub struct BCDUInt(Vec<u8>);
impl BCDUInt {
  pub fn is_zero(&self) -> bool {
    if self.0.len() == 0 {
      return true;
    }
    self.0.iter().all(|&x| x == 0)
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
      .map(|c| c.to_digit(10).unwrap() as u8)
      .collect()
  }
  pub fn from_digits(digits: Vec<u8>) -> Self {
    let mut vec = Vec::new();
    let mut i = digits.len();
    while i > 0 {
      i -= 1;
      let low = digits[i];
      let high = if i == 0 {
        0
      } else {
        i -= 1;
        digits[i]
      };
      vec.push((high << 4) | low);
    }
    vec.reverse();
    Self(vec)
  }
  fn compare_digits(a: &[u8], b: &[u8]) -> i8 {
    if a.len() > b.len() {
      1
    } else if a.len() < b.len() {
      -1
    } else {
      for (&x, &y) in a.iter().zip(b) {
        if x > y {
          return 1;
        };
        if x < y {
          return -1;
        };
      }
      0
    }
  }
  fn sub_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut carry = 0;
    let mut ai = a.len() as isize - 1;
    let mut bi = b.len() as isize - 1;

    while ai >= 0 {
      let mut ad = a[ai as usize] as i8 - carry;
      let bd = if bi >= 0 { b[bi as usize] as i8 } else { 0 };

      if ad < bd {
        ad += 10;
        carry = 1;
      } else {
        carry = 0;
      }

      result.push((ad - bd) as u8);
      ai -= 1;
      bi -= 1;
    }

    result.reverse();
    while result.first() == Some(&0) && result.len() > 1 {
      result.remove(0);
    }

    result
  }
}

impl Display for BCDUInt {
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
impl PartialEq for BCDUInt {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}

impl SubAssign<&BCDUInt> for BCDUInt {
  fn sub_assign(&mut self, rhs: &Self) {
    *self = self.clone() - rhs.clone();
  }
}

impl Add for BCDUInt {
  type Output = BCDUInt;
  fn add(self, rhs: Self) -> Self::Output {
    let lhs = self.0;
    let rhs = rhs.0;

    let mut result = Vec::new();
    let mut carry = 0;

    let max_len = lhs.len().max(rhs.len());

    for i in 0..max_len {
      let a = *lhs.get(lhs.len().wrapping_sub(1 + i)).unwrap_or(&0);
      let b = *rhs.get(rhs.len().wrapping_sub(1 + i)).unwrap_or(&0);

      // Extraer nibbles altos y bajos
      let (a1, a0) = ((a >> 4) & 0x0F, a & 0x0F);
      let (b1, b0) = ((b >> 4) & 0x0F, b & 0x0F);

      let mut sub0 = a0 + b0 + carry;
      carry = 0;
      if sub0 >= 10 {
        sub0 -= 10;
        carry = 1;
      }

      let mut sub1 = a1 + b1 + carry;
      carry = 0;
      if sub1 >= 10 {
        sub1 -= 10;
        carry = 1;
      }

      // restar 10 es suficiente y menos costoso

      result.push(((sub1 as u8) << 4) | (sub0 as u8));
    }

    // Eliminar ceros a la izquierda, excepto si es cero solo
    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }

    result.reverse();
    Self(result)
  }
}
impl Sub for BCDUInt {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    let lhs = self.0;
    let rhs = rhs.0;

    let mut result = Vec::new();
    let mut carry = 0;

    // Asumimos que lhs >= rhs comprobacion
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
    Self(result)
  }
}
impl Mul for BCDUInt {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self::Output {
    &self * &rhs
  }
}
impl Div for BCDUInt {
  type Output = Self;
  fn div(self, rhs: Self) -> Self::Output {
    let x_digits = self.digits();
    let y_digits = rhs.digits();
    if y_digits.is_empty() || y_digits.iter().all(|&d| d == 0) {
      panic!("División por cero");
    }

    let mut result = Vec::new();
    let mut remainder = Vec::new();

    for digit in x_digits {
      remainder.push(digit);
      while remainder.first() == Some(&0) && remainder.len() > 1 {
        remainder.remove(0);
      }

      let mut count = 0;
      while Self::compare_digits(&remainder, &y_digits) >= 0 {
        remainder = Self::sub_digits(&remainder, &y_digits);
        count += 1;
      }

      result.push(count);
    }

    while result.first() == Some(&0) && result.len() > 1 {
      result.remove(0);
    }

    Self::from_digits(result)
  }
}
impl Rem for BCDUInt {
  type Output = BCDUInt;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = self.clone() / rhs.clone();
    let mul = rhs * div;
    self - mul
  }
}
impl Add for &BCDUInt {
  type Output = BCDUInt;
  fn add(self, rhs: Self) -> Self::Output {
    self.clone() + rhs.clone()
  }
}
impl Sub for &BCDUInt {
  type Output = BCDUInt;

  fn sub(self, rhs: Self) -> Self::Output {
    self.clone() - rhs.clone()
  }
}
impl Mul for &BCDUInt {
  type Output = BCDUInt;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut x_digits = self.digits();
    let mut y_digits = rhs.digits();
    x_digits.reverse();
    y_digits.reverse();

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
    result.reverse();

    BCDUInt::from_digits(result)
  }
}
impl Div for &BCDUInt {
  type Output = BCDUInt;

  fn div(self, rhs: Self) -> Self::Output {
    self.clone() / rhs.clone()
  }
}
impl Rem for &BCDUInt {
  type Output = BCDUInt;
  fn rem(self, rhs: Self) -> Self::Output {
    let ref div = self / rhs;
    let ref mul = rhs * div;
    self - mul
  }
}
impl PartialOrd for BCDUInt {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.cmp(other).into()
  }
}
impl Ord for BCDUInt {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let x_digits = self.digits();
    let y_digits = other.digits();
    let x_len = x_digits.len();
    let y_len = y_digits.len();
    if x_len > y_len {
      return std::cmp::Ordering::Greater;
    } else if x_len < y_len {
      return std::cmp::Ordering::Less;
    }
    let max = x_len.max(y_len);
    for i in 0..max {
      let x = if x_len > i { x_digits[i] } else { 0 };
      let y = if y_len > i { y_digits[i] } else { 0 };
      let value = x.cmp(&y);
      if value != std::cmp::Ordering::Equal {
        return value;
      }
    }
    std::cmp::Ordering::Equal
  }
}

impl From<String> for BCDUInt {
  fn from(value: String) -> Self {
    let digits = value.chars().map(|c| c as u8 - b'0').collect();
    Self::from_digits(digits)
  }
}
impl From<u8> for BCDUInt {
  fn from(value: u8) -> Self {
    if value > 99 {
      panic!("Value must be between 0 and 99");
    }
    let high = value / 10;
    let low = value % 10;
    let byte = (high << 4) | low;
    BCDUInt(vec![byte])
  }
}
#[derive(Clone, Eq, Hash, Debug)]
pub enum BasicNumber {
  Int(bool, BCDUInt),
  Float(bool, BCDUInt, Decimals),
}
impl BasicNumber {
  pub fn is_zero(&self) -> bool {
    match self {
      Self::Int(_, x) => x.is_zero(),
      Self::Float(_, y_int, y_dec) => y_int.is_zero() && y_dec.to_string() == "0",
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
          return Self::Int(*x_neg, x_int.clone() - BCDUInt::from(1));
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
        if *x > BCDUInt::from(0) {
          return Self::Int(*x_neg, x.clone() + BCDUInt::from(1));
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
  pub fn is_int(&self) -> bool {
    match self {
      Self::Int(_, _) => true,
      Self::Float(_, _, x_dec) => x_dec.is_zero(),
    }
  }
  pub fn is_negative(&self) -> bool {
    match self {
      Self::Int(x_neg, _) => *x_neg,
      Self::Float(x_neg, _, _) => *x_neg,
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
          return Self::Int(false, BCDUInt::from(0));
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
        Self::Float(x_neg, x + y + BCDUInt(vec![carry]), z_dec)
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
          return Self::Int(false, BCDUInt::from(0));
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

        let x_full = BCDUInt::from(format!("{}{}", x, x_dec));
        let y_full = BCDUInt::from(format!("{}{}", y, y_dec));
        let (z_neg, z) = match x_full.cmp(&y_full) {
          std::cmp::Ordering::Greater => (x_neg, x_full - y_full),
          std::cmp::Ordering::Less => (!y_neg, y_full - x_full),
          std::cmp::Ordering::Equal => return Self::Int(false, BCDUInt::from(0)),
        };

        let z_str = z.to_string();
        let z_len = z_str.len();

        let (z, z_dec) = if z_len > decimals {
          let (int, dec) = z_str.split_at(z_len - decimals);
          (int.to_string(), dec.to_string())
        } else {
          (
            "0".to_string(),
            format!("{:0>width$}", z_str, width = decimals),
          )
        };

        Self::Float(z_neg, BCDUInt::from(z), Decimals::from(z_dec))
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
        let x: BCDUInt = x_str.into();
        let y_str = format!("{}{}", y, y_dec);
        let y: BCDUInt = y_str.into();
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
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        let mut x_dec = x_dec.to_string();
        let mut y_dec = y_dec.to_string();
        let max_dec = x_dec.len().max(y_dec.len());
        x_dec.extend(std::iter::repeat('0').take(max_dec + DIVISION_DECIMALS - x_dec.len()));
        y_dec.extend(std::iter::repeat('0').take(max_dec - y_dec.len()));
        let x_str = format!("{}{}", x, x_dec);
        let y_str = format!("{}{}", y, y_dec);
        let mut decimals = max_dec + DIVISION_DECIMALS - 1;

        let result = BCDUInt::from(x_str) / BCDUInt::from(y_str);
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
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), Decimals(vec![]))
          / Self::Float(false, y, Decimals(vec![]))
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
impl Rem for BasicNumber {
  type Output = BasicNumber;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = (self.clone() / rhs.clone()).trunc();
    let mul = rhs * div;
    self - mul
  }
}
impl PartialOrd for BasicNumber {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.cmp(other).into()
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
          return std::cmp::Ordering::Less;
        } else if !*x_neg && *y_neg {
          return std::cmp::Ordering::Greater;
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
#[derive(Clone, Eq, Default, Hash)]
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
  pub fn _is_zero(&self) -> bool {
    match self {
      Self::NaN | Self::Infinity | Self::NegativeInfinity => false,
      Self::Basic(x) => x.is_zero(),
      Self::Complex(x, y) => x.is_zero() && y.is_zero(),
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
          BasicNumber::Int(false, BCDUInt::from(0)),
          number,
        ))
      } else if i_exp == 2 {
        Ok(Self::Basic(-number))
      } else if i_exp == 3 {
        Ok(Self::Complex(
          BasicNumber::Int(false, BCDUInt::from(0)),
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
impl From<u128> for Number {
  fn from(value: u128) -> Self {
    Self::Basic(BasicNumber::Int(false, value.to_string().into()))
  }
}
impl From<i32> for Number {
  fn from(value: i32) -> Self {
    Self::Basic(BasicNumber::Int(
      value.is_negative(),
      value.abs().to_string().into(),
    ))
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
      (_, Self::Infinity) => Self::Basic(BasicNumber::Int(false, BCDUInt::from(0))),
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::Basic(BasicNumber::Int(false, BCDUInt::from(0))),
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
impl Rem for Number {
  type Output = Number;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = (self.clone() / rhs.clone()).trunc();
    let mul = rhs * div;
    self - mul
  }
}
impl PartialOrd for Number {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.cmp(other).into()
  }
}
impl Ord for Number {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match (self, other) {
      (Self::NaN, Self::NaN) => std::cmp::Ordering::Equal,
      (Self::NaN, _) => std::cmp::Ordering::Greater,
      (_, Self::NaN) => std::cmp::Ordering::Less,
      (Self::Infinity, _) => std::cmp::Ordering::Greater,
      (_, Self::Infinity) => std::cmp::Ordering::Less,
      (Self::NegativeInfinity, _) => std::cmp::Ordering::Less,
      (_, Self::NegativeInfinity) => std::cmp::Ordering::Greater,
      (Self::Basic(x), Self::Basic(y)) => x.cmp(y),
      (Self::Complex(a, b), Self::Complex(c, d)) => {
        let real = a.cmp(c);
        if real == std::cmp::Ordering::Equal {
          b.cmp(d)
        } else {
          real
        }
      }
      (Self::Basic(x), Self::Complex(a, b)) => {
        Self::Complex(x.clone(), BasicNumber::Int(false, BCDUInt::from(0)))
          .cmp(&Self::Complex(a.clone(), b.clone()))
      }
      (Self::Complex(a, b), Self::Basic(x)) => Self::Complex(a.clone(), b.clone()).cmp(
        &Self::Complex(x.clone(), BasicNumber::Int(false, BCDUInt::from(0))),
      ),
    }
  }
}
impl std::fmt::Debug for Number {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
impl Encode for Number {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Number as u8, StructTag::SOB as u8];

    encode.extend(
      self
        .to_string()
        .replace('\\', "\\\\") // para poder usar caracteres de control sin problemas
        .replace('\0', "\\0")
        .replace('\x01', "\\x01")
        .as_bytes(),
    );
    encode.push(StructTag::EOB as u8);

    Ok(encode)
  }
}
impl crate::Decode for Number {
  fn decode(vec: &mut std::collections::VecDeque<u8>) -> Result<Self, String> {
    vec
      .pop_front()
      .on_some_option(|byte| {
        if byte != StructTag::Number as u8 {
          None
        } else {
          Some(byte)
        }
      })
      .on_error(|_| "Se esperaba un numero")?;
    vec.pop_front(); // SOB
    let mut bytes = vec![];
    loop {
      let byte = vec.pop_front().on_error(|_| "Binario corrupto")?;
      if byte == StructTag::EOB as u8 {
        break;
      }
      bytes.push(byte);
    }
    Ok(
      String::from_utf8_lossy(&bytes)
        .to_string()
        .parse::<Self>()?,
    )
  }
}
