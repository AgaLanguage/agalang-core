use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

use crate::{
  compiler::value::number::traits::{AsNumber as _, FromStrRadix},
  util::{OnError, OnSome},
  StructTag,
};

mod binary;
pub mod traits;
pub use binary::Big256 as BigUInt;
mod float;
pub use float::BigUDecimal as BigUFloat;

mod real;
use real::RealNumber;

const NAN_NAME: &str = "NeN";
const INFINITY_NAME: &str = "infinito";

pub enum NumberError {
  Empty,
  Radix(u8),
  InvalidCharacter(char),
  InvalidDigit(char, u8),
}
impl std::fmt::Debug for NumberError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let data = match &self {
      Self::Empty => "Cannot parse empty string".to_string(),
      Self::Radix(radix) => format!("Invalid radix {}", radix),
      Self::InvalidCharacter(c) => format!("Invalid character '{}'", c),
      Self::InvalidDigit(c, radix) => format!("Invalid digit '{}' for base {}", c, radix),
    };
    write!(f, "{data}")
  }
}
macro_rules! op_number_real_complex {
  ($num:expr, $op:ident) => {
    match $num {
      Number::NaN => Number::NaN,
      Number::Infinity => Number::Infinity,
      Number::NegativeInfinity => Number::NegativeInfinity,
      Number::Real(x) => Number::Real(x.$op()),
      Number::Complex(x, y) => {
        let x = x.$op();
        let y = y.$op();
        if y.is_zero() {
          Number::Real(x)
        } else {
          Number::Complex(x, y)
        }
      }
    }
  };
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Eq, Default, Hash)]
pub enum Number {
  #[default]
  NaN,
  Infinity,
  NegativeInfinity,
  Real(RealNumber),
  Complex(RealNumber, RealNumber),
}
impl Number {
  pub fn normalize(&mut self) {
    if let Self::Real(real) = self {
      real.normalize();
      return;
    }
    if let Self::Complex(real, imaginary) = self {
      real.normalize();
      if imaginary.is_zero() {
        let new_real = std::mem::take(real);
        *self = Self::Real(new_real);
        return;
      }
      imaginary.normalize();
    }
  }
  fn into_normalize(mut self) -> Self {
    self.normalize();
    self
  }
  pub fn ceil(&self) -> Self {
    op_number_real_complex!(self, ceil)
  }
  pub fn floor(&self) -> Self {
    op_number_real_complex!(self, floor)
  }
  pub fn trunc(&self) -> Self {
    op_number_real_complex!(self, trunc)
  }
  pub fn round(&self) -> Self {
    op_number_real_complex!(self, round)
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
      Self::Real(x) => x.is_zero(),
      Self::Complex(x, y) => x.is_zero() && y.is_zero(),
    }
  }
  pub fn pow(&self, exp: Self) -> Self {
    // TODO: implementar correctamente las potencias. Esta implementacion es muy basica.
    match (self, exp) {
      (Self::NaN, _) | (_, Self::NaN) => Self::NaN,
      (Self::Infinity, Self::Real(e)) | (Self::NegativeInfinity, Self::Real(e)) => {
        if e.is_negative() || e.is_zero() {
          return Self::Real(RealNumber::Int(false, BigUInt::from(0u8)));
        }
        if e.is_int() {
          if let RealNumber::Int(_, e) = e {
            if e.unit() % 2 == 0 {
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
      (Self::Real(x), Self::Real(y)) => {
        if x.is_zero() && y.is_negative() {
          return Self::Infinity;
        }
        if x.is_zero() && y.is_zero() {
          return Self::NaN;
        }
        if x.is_zero() {
          return Self::Real(RealNumber::Int(false, BigUInt::from(0u8)));
        }
        if y.is_zero() {
          return Self::Real(RealNumber::Int(false, BigUInt::from(1u8)));
        }
        if !y.is_int() {
          todo!("No se ha implementado la potencia x^y cuando y no es entero")
        }
        if let RealNumber::Int(y_neg, y) = y {
          let mut result = RealNumber::Int(false, BigUInt::from(1u8));
          let mut base = x.clone();
          let mut exponent = y;
          while !exponent.is_zero() {
            if exponent.unit() & 1 == 1 {
              result = &result * &base;
            }
            base = &base * &base;
            exponent /= &BigUInt::from(2u8);
          }
          if y_neg {
            return Self::Real(RealNumber::Float(false, BigUFloat::default()));
          }
          return Self::Real(result);
        }
        Self::NaN
      }
      (_, _) => Self::NaN,
    }
  }
}
impl<T> From<T> for Number
where
  T: traits::ToDigits,
{
  fn from(value: T) -> Self {
    Self::Real(RealNumber::Int(false, value.into()))
  }
}
impl From<i32> for Number {
  fn from(value: i32) -> Self {
    Self::Real(RealNumber::Int(
      value.is_negative(),
      value.unsigned_abs().into(),
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
      Self::Real(x) => write!(f, "{x}"),
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

// TODO: Validar operaciones. Tengo mis dudas.
impl Add for &Number {
  type Output = Number;
  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Number::NaN, _) => Number::NaN,
      (_, Number::NaN) => Number::NaN,
      (Number::Infinity, _) => Number::Infinity,
      (_, Number::Infinity) => Number::Infinity,
      (Number::NegativeInfinity, _) => Number::NegativeInfinity,
      (_, Number::NegativeInfinity) => Number::NegativeInfinity,
      (Number::Real(x), Number::Real(y)) => Number::Real(x + y),
      (Number::Complex(x, y), Number::Complex(a, b)) => Number::Complex(x + a, y + b),
      (Number::Real(x), Number::Complex(a, b)) => Number::Complex(x + a, b.clone()),
      (Number::Complex(a, b), Number::Real(x)) => Number::Complex(a + x, b.clone()),
    }
    .into_normalize()
  }
}
impl Sub for &Number {
  type Output = Number;
  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Number::NaN, _) => Number::NaN,
      (_, Number::NaN) => Number::NaN,
      (_, Number::Infinity) => Number::NegativeInfinity,
      (Number::NegativeInfinity, _) => Number::NegativeInfinity,
      (Number::Infinity, _) => Number::Infinity,
      (_, Number::NegativeInfinity) => Number::Infinity,
      (Number::Real(x), Number::Real(y)) => Number::Real(x - y),
      (Number::Complex(x, y), Number::Complex(a, b)) => Number::Complex(x - a, y - b),
      (Number::Real(x), Number::Complex(a, b)) => Number::Complex(x - a, -b.clone()),
      (Number::Complex(a, b), Number::Real(x)) => Number::Complex(a - x, b.clone()),
    }
    .into_normalize()
  }
}
impl Mul for &Number {
  type Output = Number;
  fn mul(self, rhs: Self) -> Self::Output {
    let (a, b, c, d) = match (self, rhs) {
      (Number::NaN, _) | (_, Number::NaN) => return Number::NaN,
      (Number::Infinity, _) => return Number::Infinity,
      (_, Number::Infinity) => return Number::Infinity,
      (Number::NegativeInfinity, _) => return Number::NegativeInfinity,
      (_, Number::NegativeInfinity) => return Number::NegativeInfinity,
      (Number::Real(x), Number::Real(y)) => return Number::Real(x * y),
      (Number::Complex(a, b), Number::Complex(c, d)) => (a, b, c, d),
      (Number::Real(x), Number::Complex(a, b)) => (x, &Default::default(), a, b),
      (Number::Complex(a, b), Number::Real(x)) => (a, b, x, &Default::default()),
    };
    Number::Complex(&(a * c) - &(b * d), &(a * d) + &(c * b)).into_normalize()
  }
}
impl Div for &Number {
  type Output = Number;
  fn div(self, rhs: Self) -> Self::Output {
    let (a, b, c, d) = match (self, rhs) {
      (Number::NaN, _) | (_, Number::NaN) => return Number::NaN,
      (Number::Infinity, _) => return Number::Infinity,
      (_, Number::Infinity) => return Number::Real(RealNumber::Int(false, BigUInt::from(0u8))),
      (Number::NegativeInfinity, _) => return Number::NegativeInfinity,
      (_, Number::NegativeInfinity) => {
        return Number::Real(RealNumber::Int(false, BigUInt::from(0u8)))
      }
      (Number::Real(x), Number::Real(y)) => return Number::Real(x / y),
      (Number::Complex(a, b), Number::Complex(c, d)) => (a, b, c, d),
      (Number::Real(x), Number::Complex(a, b)) => (x, &Default::default(), a, b),
      (Number::Complex(a, b), Number::Real(x)) => (a, b, x, &Default::default()),
    };
    let conj = &(&(c * c) + &(d * d));

    Number::Complex(&(&(a * c) + &(b * d)) / conj, &(&(b * c) - &(a * d)) / conj).into_normalize()
  }
}
impl Neg for Number {
  type Output = Self;
  fn neg(self) -> Self::Output {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity => Self::NegativeInfinity,
      Self::NegativeInfinity => Self::Infinity,
      Self::Real(x) => Self::Real(-x),
      Self::Complex(x, y) => Self::Complex(-x, -y),
    }
  }
}
impl Rem for &Number {
  type Output = Number;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = (self / rhs).trunc();
    let mul = rhs * &div;
    self - &mul
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
      (Self::Real(x), Self::Real(y)) => x.cmp(y),
      (Self::Complex(a, b), Self::Complex(c, d)) => {
        let real = a.cmp(c);
        if real == std::cmp::Ordering::Equal {
          b.cmp(d)
        } else {
          real
        }
      }
      (Self::Real(x), Self::Complex(a, b)) => {
        Self::Complex(x.clone(), RealNumber::Int(false, BigUInt::from(0u8)))
          .cmp(&Self::Complex(a.clone(), b.clone()))
      }
      (Self::Complex(a, b), Self::Real(x)) => Self::Complex(a.clone(), b.clone()).cmp(
        &Self::Complex(x.clone(), RealNumber::Int(false, BigUInt::from(0u8))),
      ),
    }
  }
}
impl std::fmt::Debug for Number {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}") // usa Display
  }
}
impl std::str::FromStr for Number {
  type Err = NumberError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use traits::FromStrRadix;
    Self::from_str_radix(s, 10)
  }
}
impl FromStrRadix for Number {
  fn from_str_radix(value: &str, radix: u8) -> Result<Self, NumberError> {
    let real = RealNumber::from_str_radix(value, radix)?;
    let number = Self::Real(real);
    Ok(number)
  }
}
impl crate::Encode for Number {
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
    String::from_utf8_lossy(&bytes)
      .as_radix(10)
      .map_err(|e| format!("{e:?}"))
  }
}
