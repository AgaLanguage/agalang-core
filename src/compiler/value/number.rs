use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
  str::FromStr,
};

use crate::{
  util::{OnError, OnSome},
  Encode, StructTag,
};

mod binary;
pub use binary::BCDUInt as BigUInt;

const DIVISION_DECIMALS: usize = 100;
const NAN_NAME: &str = "NeN";
const INFINITY_NAME: &str = "infinito";

#[derive(Clone, Eq, Hash, Debug)]
#[allow(clippy::derived_hash_with_manual_eq)]
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
    Some(self.cmp(other))
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

fn add_float((x_neg, x, x_dec):(&bool,&BigUInt, &Decimals), (y_neg, y, y_dec):(&bool,&BigUInt, &Decimals)) -> (bool, BigUInt, Decimals){
  if !x_neg && *y_neg {
          return sub_float((&false, y, y_dec), (&false, x, x_dec))
        } else if *x_neg && !y_neg {
          return sub_float((&false, x, x_dec), (&false, y, y_dec))
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
      result.push(sum);
    }
    result.reverse();
    Decimals(result)
  };
  (*x_neg, &(x + y) + &BigUInt::from_digits(vec![carry]), z_dec)
}
fn sub_float((x_neg, x, x_dec):(&bool, &BigUInt, &Decimals), (y_neg, y, y_dec):(&bool, &BigUInt, &Decimals)) -> (bool, BigUInt, Decimals){
          if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          let (z_neg, z, z_dec) = add_float((&false, x, x_dec), (&false, y, y_dec));
          return (z_neg, z, z_dec)
        }
  let mut x_dec = x_dec.to_string();
   let mut y_dec = y_dec.to_string();
   let decimals = x_dec.len().max(y_dec.len());
   x_dec.extend(std::iter::repeat_n('0', decimals - x_dec.len()));
   y_dec.extend(std::iter::repeat_n('0', decimals - y_dec.len()));

   let x_full = BigUInt::from(format!("{}{}", x, x_dec));
   let y_full = BigUInt::from(format!("{}{}", y, y_dec));
   let (z_neg, z) = match x_full.cmp(&y_full) {
     std::cmp::Ordering::Greater => (*x_neg, &x_full - &y_full),
     std::cmp::Ordering::Less => (!y_neg, &y_full - &x_full),
     std::cmp::Ordering::Equal => return (false, BigUInt::from(0), Decimals(vec![])),
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

   (z_neg, BigUInt::from(z), Decimals::from(z_dec))
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Eq, Debug, Hash)]
pub enum BasicNumber {
  Int(bool, BigUInt),
  Float(bool, BigUInt, Decimals),
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
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x_int, x_dec) => {
        if x_dec.to_string() == "0" {
          return Self::Int(*x_neg, x_int.clone());
        }
        if *x_neg {
          return Self::Int(*x_neg, x_int - &BigUInt::from(1));
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
        if *x > BigUInt::from(0) {
          return Self::Int(*x_neg, x + &BigUInt::from(1));
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
            return Self::Int(*x_neg, x + &1.into());
          }
          if digit < 5 {
            return Self::Int(*x_neg, x - &1.into());
          }
        }
        // si el decimal es 0.5 se redondea al par más cercano
        Self::Int(
          *x_neg,
          x
            + &if x.last() % 2 == 0 {
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
impl Display for BasicNumber {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Int(x_neg, x) => write!(f, "{}{}", if *x_neg { "-" } else { "" }, x),
      Self::Float(x_neg, x, x_dec) => write!(f, "{}{}.{}", if *x_neg { "-" } else { "" }, x, x_dec),
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

impl Add for &BasicNumber {
  type Output = BasicNumber;
  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => {
        if x_neg == y_neg {
          return BasicNumber::Int(*x_neg, x + y);
        }
        if (x.is_zero() && y.is_zero()) || (x == y) {
          return BasicNumber::Int(false, BigUInt::from(0));
        }
        if y > x {
          return BasicNumber::Int(*y_neg, y - x);
        }
        BasicNumber::Int(*x_neg, x - y)
      }
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Float(y_neg, y, y_dec)) => {
        let (z_neg,z, z_dec) = add_float((x_neg,x, x_dec), (y_neg,y, y_dec));
        BasicNumber::Float(z_neg, z, z_dec)
      }
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y, y_dec)) => {
        let (z_neg,z, z_dec) = add_float((x_neg,x, &Decimals(vec![])), (y_neg,y, y_dec));
        BasicNumber::Float(z_neg, z, z_dec)
      }
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Int(y_neg, y)) => {
        let (z_neg,z, z_dec) = add_float((x_neg,x, x_dec), (y_neg,y, &Decimals(vec![])));
        BasicNumber::Float(z_neg, z, z_dec)
      }
    }
  }
}
impl Sub for &BasicNumber {
  type Output = BasicNumber;

  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return BasicNumber::Int(*x_neg, x + y);
        }
        if x == y {
          return BasicNumber::Int(false, BigUInt::from(0));
        }
        if x > y {
          BasicNumber::Int(*x_neg, x - y)
        } else {
          BasicNumber::Int(!y_neg, y - x)
        }
      }

      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Float(y_neg, y, y_dec)) => {
        let (z_neg, z, z_dec) = sub_float((x_neg, x, x_dec), (y_neg, y, y_dec));
        BasicNumber::Float(z_neg, z, z_dec)
      }

      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y, y_dec)) => {
        let (z_neg, z, z_dec) = sub_float((x_neg, x, &Decimals(vec![])), (y_neg, y, y_dec));
        BasicNumber::Float(z_neg, z, z_dec)
      }
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Int(y_neg, y)) => {
        let (z_neg, z, z_dec) = sub_float((x_neg, x, x_dec), (y_neg, y, &Decimals(vec![])));
        BasicNumber::Float(z_neg, z, z_dec)
      }
    }
  }
}
impl Mul for &BasicNumber {
  type Output = BasicNumber;
  fn mul(self, rhs: Self) -> Self::Output {
    let (neg, (x, x_dec), (y, y_dec)) = match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => return BasicNumber::Int(x_neg ^ y_neg, x * y),
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Float(y_neg, y, y_dec)) => (x_neg ^ y_neg,(x, x_dec), (y, y_dec)),
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y, y_dec)) => 
        (x_neg ^ y_neg,( x, &Decimals(vec![])),( y, y_dec)),
      
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Int(y_neg, y)) => 
        (x_neg ^ y_neg, (x, x_dec),( y, &Decimals(vec![]))),
    };
            let x_dec = x_dec.to_string();
        let y_dec = y_dec.to_string();
        let x_str = format!("{}{}", x, x_dec);
        let x: BigUInt = x_str.into();
        let y_str = format!("{}{}", y, y_dec);
        let y: BigUInt = y_str.into();
        let mut decimals = x_dec.len() + y_dec.len();

        let result = &x * &y;
        let result = result.to_string();

        let mut frac_part = String::new();
        let mut number = result.chars().collect::<Vec<_>>();
        while decimals > 0 {
          decimals -= 1;
          frac_part.push(number.pop().unwrap_or('0'));
        }
        let int_part = if number.is_empty() {
          "0".to_string()
        } else {
          number.iter().collect()
        };

        let frac_part = frac_part.chars().rev().collect::<String>();

        BasicNumber::Float(neg, int_part.into(), frac_part.into())
  }
}
impl Div for &BasicNumber {
  type Output = BasicNumber;
  fn div(self, rhs: Self) -> Self::Output {
    let (neg, (x, x_dec), (y, y_dec)) = match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => (x_neg ^ y_neg, (x, &Decimals(vec![])),( y, &Decimals(vec![]))),
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Float(y_neg, y, y_dec)) => (x_neg ^ y_neg,(x, x_dec), (y, y_dec)),
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y, y_dec)) => 
        (x_neg ^ y_neg,( x, &Decimals(vec![])),( y, y_dec)),
      
      (BasicNumber::Float(x_neg, x, x_dec), BasicNumber::Int(y_neg, y)) => 
        (x_neg ^ y_neg, (x, x_dec),( y, &Decimals(vec![]))),
    };
    let mut x_dec = x_dec.to_string();
        let mut y_dec = y_dec.to_string();
        let max_dec = x_dec.len().max(y_dec.len());
        x_dec.extend(std::iter::repeat_n(
          '0',
          max_dec + DIVISION_DECIMALS - x_dec.len(),
        ));
        y_dec.extend(std::iter::repeat_n('0', max_dec - y_dec.len()));
        let x_str = format!("{}{}", x, x_dec);
        let y_str = format!("{}{}", y, y_dec);
        let mut decimals = max_dec + DIVISION_DECIMALS - 1;

        let result = &BigUInt::from(x_str) / &BigUInt::from(y_str);
        let result = result.to_string();

        let mut frac_part = String::new();
        let mut number = result.chars().collect::<Vec<_>>();
        while decimals > 0 {
          decimals -= 1;
          frac_part.push(number.pop().unwrap_or('0'));
        }
        let int_part = if number.is_empty() {
          "0".to_string()
        } else {
          number.iter().collect()
        };

        let frac_part = frac_part.chars().rev().collect::<String>();

        BasicNumber::Float(neg, int_part.into(), frac_part.into())
    }
}
impl Neg for BasicNumber {
  type Output = BasicNumber;
  fn neg(self) -> Self::Output {
    match self {
      BasicNumber::Int(x_neg, x) => BasicNumber::Int(!x_neg, x),
      BasicNumber::Float(x_neg, x, x_dec) => BasicNumber::Float(!x_neg, x, x_dec),
    }
  }
}
impl Rem for &BasicNumber {
  type Output = BasicNumber;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = (self / rhs).trunc();
    let mul = rhs * &div;
    self - &mul
  }
}
impl PartialOrd for BasicNumber {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
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
          return x.cmp(y).reverse();
        }
        x.cmp(y)
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
        x_int.cmp(y_int)
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
#[allow(clippy::derived_hash_with_manual_eq)]
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
  pub fn is_zero(&self) -> bool {
    match self {
      Self::NaN | Self::Infinity | Self::NegativeInfinity => false,
      Self::Basic(x) => x.is_zero(),
      Self::Complex(x, y) => x.is_zero() && y.is_zero(),
    }
  }
  pub fn from_str_radix(value: &str, radix: u8) -> Self {
    if !(2..=36).contains(&radix) {
      return Self::NaN;
    }
    // TODO: Independizar de i32
    i32::from_str_radix(value, radix as u32)
      .map(|v| v.to_string().parse::<Self>().unwrap_or_default())
      .unwrap_or_default()
    }
    pub fn pow(&self, exp: Self) -> Self {
    // TODO: implementar correctamente las potencias. Esta implementacion es muy basica.
    match (self, exp) {
      (Self::NaN, _) | (_, Self::NaN) => Self::NaN,
      (Self::Infinity, Self::Basic(e)) | (Self::NegativeInfinity, Self::Basic(e)) => {
        if e.is_negative() || e.is_zero() {
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(0)));
        }
        if e.is_int() {
          if let BasicNumber::Int(_, e) = e {
            if e.last() % 2 == 0 {
              return Self::Infinity;
            } else if matches!(self, Self::NegativeInfinity) {
              return Self::NegativeInfinity;
            } else {
              return Self::Infinity;
            }
          }
        }
        Self::Infinity
      }
      (Self::Infinity, _) | (Self::NegativeInfinity, _) => Self::NaN,
      (Self::Basic(x), Self::Basic(e)) => {
        if x.is_zero() && e.is_negative() {
          return Self::Infinity;
        }
        if x.is_zero() && e.is_zero() {
          return Self::NaN;
        }
        if x.is_zero() {
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(0)));
        }
        if e.is_zero() {
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(1)));
        }
        if !e.is_int() {
          return Self::NaN;
        }
        if let BasicNumber::Int(e_neg, e) = e {
          let mut result = BasicNumber::Int(false, BigUInt::from(1));
          let mut base = x.clone();
          let mut exponent = e;
          while !exponent.is_zero() {
            if exponent.last() % 2 == 1 {
              result = &result * &base;
            }
            base = &base * &base;
            exponent = &exponent / &BigUInt::from(2);
          }
          if e_neg {
            return Self::Basic(BasicNumber::Float(
              false,
              BigUInt::from(0),
              result.to_string().into(),
            ));
          }
          return Self::Basic(result);
        }
        Self::NaN
      }
      (_, _) => Self::NaN,
    }
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
    // TODO: Verificar la validez, no estoy seguro de haberlo implementado bien. Algo me dice que no.
    if s.ends_with("i") {
      let parts: Vec<&str> = s.split("i").collect();
      let i_exp = parts.len() - 1;
      let number = parts.join("").into();
      for (i, part) in parts.iter().enumerate() {
        if i == 0 {
          continue;
        }
        if part
          .chars()
          .any(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.')
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
          BasicNumber::Int(false, BigUInt::from(0)),
          number,
        ))
      } else if i_exp == 2 {
        Ok(Self::Basic(-number))
      } else if i_exp == 3 {
        Ok(Self::Complex(
          BasicNumber::Int(false, BigUInt::from(0)),
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
    if s.chars().all(|c| c.is_ascii_digit()) {
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
impl From<u16> for Number {
  fn from(value: u16) -> Self {
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
impl From<Number> for Result<usize, String> {
  fn from(value: Number) -> Self {
    let string = value.floor().to_string();
    string
      .parse::<usize>()
      .on_error(|_| format!("No se puede convertir el número '{}' a usize", string))
  }
}
impl Display for Number {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NaN => write!(f, "{NAN_NAME}"),
      Self::Infinity => write!(f, "{INFINITY_NAME}"),
      Self::NegativeInfinity => write!(f, "-{INFINITY_NAME}"),
      Self::Basic(x) => write!(f, "{x}"),
      Self::Complex(x, y) => {
        if x.is_zero() && y.is_zero() {
          write!(f, "0")
        } else if x.is_zero() {
          write!(f, "{y}i")
        } else if y.is_zero() {
          write!(f, "{x}")
        } else {
          write!(f, "{x} + {y}i")
        }
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
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(&x + &y),
      (Self::Complex(x, y), Self::Complex(a, b)) => Self::Complex(&x + &a, &y + &b),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&x + &a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&a + &x, b)
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
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(&x - &y),
      (Self::Complex(x, y), Self::Complex(a, b)) => Self::Complex(&x - &a, &y - &b),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&x - &a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&a - &x, b)
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
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(&x * &y),
      (Self::Complex(a, b), Self::Complex(c, d)) => Self::Complex(
        &(&a * &c) - &(&b * &d),
        &(&a * &d) + &(&c * &b),
      ),
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&x * &a, &x * &b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&a * &x, &b * &x)
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
      (_, Self::Infinity) => Self::Basic(BasicNumber::Int(false, BigUInt::from(0))),
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::Basic(BasicNumber::Int(false, BigUInt::from(0))),
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(&x / &y),
      (Self::Complex(ref a, ref b), Self::Complex(ref c, ref d)) => {
        let conj = &(&(c * c) + &(d * d));

        Self::Complex(
          &(&(a * c) + &(b * d)) / conj,
          &(&(b * c) - &(a * d)) / conj,
        )
      }
      (Self::Basic(x), Self::Complex(a, b)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&x / &a, b)
      }
      (Self::Complex(a, b), Self::Basic(x)) => {
        if x.is_zero() {
          return Self::Complex(a, b);
        }
        if a.is_zero() && b.is_zero() {
          return Self::Basic(x);
        }
        Self::Complex(&a / &x, b)
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
    Some(self.cmp(other))
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
        Self::Complex(x.clone(), BasicNumber::Int(false, BigUInt::from(0)))
          .cmp(&Self::Complex(a.clone(), b.clone()))
      }
      (Self::Complex(a, b), Self::Basic(x)) => Self::Complex(a.clone(), b.clone()).cmp(
        &Self::Complex(x.clone(), BasicNumber::Int(false, BigUInt::from(0))),
      ),
    }
  }
}
impl std::fmt::Debug for Number {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}") // usa Display
  }
}
impl Encode for Number {
  fn encode(&self) -> Result<Vec<u8>, String> {
    let mut encode = vec![StructTag::Number as u8];

    encode.extend(
      self
        .to_string()
        .replace('\\', "\\\\") // para poder usar caracteres de control sin problemas
        .replace('\0', "\\0")
        .replace('\x01', "\\x01")
        .as_bytes(),
    );
    encode.push(StructTag::EndOfBlock as u8);

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
    let mut bytes = vec![];
    loop {
      let byte = vec.pop_front().on_error(|_| "Binario corrupto")?;
      if byte == StructTag::EndOfBlock as u8 {
        break;
      }
      bytes.push(byte);
    }
    String::from_utf8_lossy(&bytes).to_string().parse::<Self>()
  }
}
