#![allow(dead_code)]
use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Sub, SubAssign},
  str::FromStr,
  sync::LazyLock,
};

const DIVISION_DECIMALS: usize = 100;

const DECIMALS_ZERO: Decimals = Decimals(vec![]);
const DECIMALS_MIDDLE: LazyLock<Decimals> = LazyLock::new(|| Decimals(vec![5 << 4]));

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
    if self.0.len() != other.0.len() {
      return false;
    }
    for i in 0..self.0.len() {
      if self.0.get(i).unwrap_or(&0) != other.0.get(i).unwrap_or(&0) {
        return false;
      }
    }
    true
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct UInt(Vec<u8>);
impl UInt {
  pub fn is_zero(&self) -> bool {
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
}

impl Display for UInt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut result = String::new();
    for i in 0..self.0.len() {
      let byte = self.0[i];
      let (byte_1, byte_0) = ((byte & 0xF0) >> 4, byte & 0x0F);
      result.push_str(&format!("{:X}{:X}", byte_1, byte_0));
    }
    write!(f, "{}", result.trim_start_matches('0'))
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
    let self_vec = self.clone().0.iter().rev().cloned().collect::<Vec<_>>();
    let rhs_vec = rhs.clone().0.iter().rev().cloned().collect::<Vec<_>>();
    let mut carry = 0;
    let mut result = Vec::new();
    let max_len = self_vec.len().max(rhs_vec.len());
    for i in 0..max_len {
      let i = max_len - 1 - i;
      let a = *self_vec.get(i).unwrap_or(&0);
      let b = *rhs_vec.get(i).unwrap_or(&0);

      let (a_byte_1, a_byte_0) = ((a & 0xF0) >> 4, a & 0x0F);
      let (b_byte_1, b_byte_0) = ((b & 0xF0) >> 4, b & 0x0F);

      let b_byte_0_c = b_byte_0 + carry;
      let sub_0 = if a_byte_0 < b_byte_0_c {
        carry = 1;
        b_byte_0_c - a_byte_0
      } else {
        carry = 0;
        a_byte_0 - b_byte_0_c
      };
      let b_byte_1_c = b_byte_1 + carry;
      let sub_1 = if a_byte_1 < b_byte_1_c {
        carry = 1;
        b_byte_1_c - a_byte_1
      } else {
        carry = 0;
        a_byte_1 - b_byte_1_c
      };
      result.push(sub_1 << 4 | sub_0);
    }
    result.reverse();
    UInt(result)
  }
}
impl Mul for &UInt {
  type Output = UInt;
  fn mul(self, rhs: Self) -> Self::Output {
    let mut result = UInt::from(0);
    let x_vec = self.clone().to_string().chars().rev().collect::<String>();
    let y_vec = rhs.clone().to_string().chars().rev().collect::<String>();

    for i in 0..x_vec.len() {
      let x = x_vec.chars().nth(i).unwrap_or('0') as u8 - b'0';
      if x > 9 {
        break;
      }
      for j in 0..y_vec.len() {
        let y = y_vec.chars().nth(j).unwrap_or('0') as u8 - b'0';
        if y > 9 {
          break;
        }
        result =
          result + format!("{}{}", x * y, String::from_utf8(vec![b'0'; i + j]).unwrap()).into();
      }
    }
    result
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

    println!("Dividend: {:?}", self.num_digits());

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum BasicNumber {
  Int(bool, UInt),
  Float(bool, UInt, Decimals),
}
impl BasicNumber {
  pub fn is_zero(&self) -> bool {
    match self {
      Self::Int(_, x) => x.is_zero(),
      Self::Float(_, y_int, y_dec) => y_int.is_zero() && y_dec == &DECIMALS_ZERO,
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
        if x_dec == &DECIMALS_ZERO {
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
        if *x_dec > DECIMALS_MIDDLE.clone() {
          return Self::Int(*x_neg, x.clone() + UInt::from(1));
        }
        if *x_dec < DECIMALS_MIDDLE.clone() {
          return Self::Int(*x_neg, x.clone() - UInt::from(1));
        }
        // si el decimal es 0.5 se redondea al par más cercano
        Self::Int(
          *x_neg,
          x.clone()
            + if x.0.last().unwrap_or(&0) % 2 == 0 {
              UInt::from(0)
            } else {
              UInt::from(1)
            },
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
          DECIMALS_ZERO.clone()
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
        Self::Float(x_neg, x.clone(), DECIMALS_ZERO.clone()) + Self::Float(y_neg, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg, x.clone(), x_dec.clone()) + Self::Float(y_neg, y, DECIMALS_ZERO.clone())
      }
    }
  }
}
impl Sub for BasicNumber {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => {
        if x_neg == y_neg {
          return Self::Int(x_neg, x - y);
        }
        if (x.is_zero() && y.is_zero()) || (x == y) {
          return Self::Int(false, UInt::from(0));
        }
        if y > x {
          return Self::Int(y_neg, y - x);
        }
        return Self::Int(x_neg, x + y);
      }
      (Self::Float(x_neg, x, x_dec), Self::Float(y_neg, y, y_dec)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y
          // -x -y = -(x + y)
          // El signo de x siempre es igual al signo comun
          return Self::Float(x_neg, x, x_dec) + Self::Float(x_neg, y, y_dec);
        }
        let (signe, (x, x_dec), (y, y_dec)) = if x > y {
          (x_neg, (x, x_dec), (y, y_dec))
        } else if y > x {
          (!y_neg, (y, y_dec), (x, x_dec))
        } else if x_dec > y_dec {
          (x_neg, (x, x_dec), (y, y_dec))
        } else if y_dec > x_dec {
          (!y_neg, (y, y_dec), (x, x_dec))
        } else {
          return Self::Int(false, UInt::from(0));
        };
        if y.is_zero() && y_dec.is_zero() {
          return Self::Float(signe, x, x_dec);
        }
        if y_dec.is_zero() {
          return Self::Float(signe, x - y, x_dec);
        }

        let mut carry = if x_dec < y_dec { 1 } else { 0 };
        let z_dec = if x_dec.is_zero() && y_dec.is_zero() {
          DECIMALS_ZERO.clone()
        } else if y_dec.is_zero() {
          x_dec.clone()
        } else {
          let mut result = Vec::new();
          let x_vec = x_dec.0.iter().rev().cloned().collect::<Vec<_>>();
          let y_vec = y_dec.0.iter().rev().cloned().collect::<Vec<_>>();
          let max_len = x_vec.len().max(y_vec.len());
          for i in 0..max_len {
            let a = *x_vec.get(i).unwrap_or(&0);
            let b = *y_vec.get(i).unwrap_or(&0);
            let (a_byte_1, a_byte_0) = ((a & 0xF0) >> 4, a & 0x0F);
            let (b_byte_1, b_byte_0) = ((b & 0xF0) >> 4, b & 0x0F);

            let b_byte_0_c = b_byte_0 + carry;
            let sub_0 = if a_byte_0 < b_byte_0_c {
              carry = 1;
              b_byte_0_c - a_byte_0
            } else {
              carry = 0;
              a_byte_0 - b_byte_0_c
            };
            let b_byte_1_c = b_byte_1 + carry;
            let sub_1 = if a_byte_1 < b_byte_1_c {
              carry = 1;
              b_byte_1_c - a_byte_1
            } else {
              carry = 0;
              a_byte_1 - b_byte_1_c
            };
            result.push(sub_1 << 4 | sub_0);
          }
          result.reverse();
          Decimals(result)
        };

        Self::Float(signe, x - y - UInt(vec![carry]), z_dec)
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg, x.clone(), DECIMALS_ZERO.clone()) - Self::Float(y_neg, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg, x.clone(), x_dec.clone()) - Self::Float(y_neg, y, DECIMALS_ZERO.clone())
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
        let y_str = format!("{}{}", y, y_dec);
        let decimals = x_dec.len() + y_dec.len() - 1;

        let result = UInt::from(x_str) * y_str.into();
        let result = result.to_string();

        let mut int_part = String::new();
        let mut frac_part = String::new();
        for (i, c) in result.chars().enumerate() {
          if decimals > result.len() {
            frac_part.push(c);
          } else if i <= (result.len() - decimals) {
            int_part.push(c);
          } else {
            frac_part.push(c);
          }
        }

        Self::Float(x_neg ^ y_neg, int_part.into(), frac_part.into())
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), DECIMALS_ZERO.clone()) * Self::Float(false, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), x_dec.clone())
          * Self::Float(false, y, DECIMALS_ZERO.clone())
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
        let decimals = DIVISION_DECIMALS - 1;

        let result = UInt::from(x_str) / UInt::from(y_str);
        println!("Result: {}", result);
        let result = result.to_string();

        let mut int_part = String::new();
        let mut frac_part = String::new();
        for (i, c) in result.chars().enumerate() {
          if decimals > result.len() {
            frac_part.push(c);
          } else if i <= (result.len() - decimals) {
            int_part.push(c);
          } else {
            frac_part.push(c);
          }
        }

        Self::Float(x_neg ^ y_neg, int_part.into(), frac_part.into())
      }
      (Self::Int(x_neg, x), Self::Float(y_neg, y, y_dec)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), DECIMALS_ZERO.clone()) / Self::Float(false, y, y_dec)
      }
      (Self::Float(x_neg, x, x_dec), Self::Int(y_neg, y)) => {
        Self::Float(x_neg ^ y_neg, x.clone(), x_dec.clone())
          / Self::Float(false, y, DECIMALS_ZERO.clone())
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
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
      .map(|v| Self::from(v))
      .unwrap_or(Self::NaN)
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
impl From<f64> for Number {
  fn from(value: f64) -> Self {
    if value.is_nan() {
      return Self::NaN;
    }
    if value.is_infinite() {
      if value.is_sign_positive() {
        return Self::Infinity;
      } else {
        return Self::NegativeInfinity;
      }
    }
    let binding = value.abs().to_string();
    let split = binding.split('.');
    let int_part = split.clone().nth(0).unwrap_or("0").to_string();
    let frac_part = split.clone().nth(1).unwrap_or("0").to_string();
    Self::Basic(BasicNumber::Float(
      value.is_sign_negative(),
      int_part.into(),
      frac_part.into(),
    ))
  }
}
impl From<&char> for Number {
  fn from(value: &char) -> Self {
    Self::Basic(BasicNumber::Int(false, value.to_string().into()))
  }
}
impl FromStr for Number {
  type Err = ();
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
    if let Ok(value) = s.parse::<i32>() {
      return Ok(Self::Basic(BasicNumber::Int(
        value.is_negative(),
        value.abs().to_string().into(),
      )));
    }
    if let Ok(value) = s.parse::<f64>() {
      return Ok(Self::from(value));
    }
    Err(())
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
