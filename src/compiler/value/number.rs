use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
  str::FromStr,
};

use crate::{
  util::{OnError, OnSome},
  Encode, StructTag,
};

mod traits;
mod binary;
pub use binary::Big256 as BigUInt;
mod float;
pub use float::BigUDecimal as BigUFloat;

const NAN_NAME: &str = "NeN";
const INFINITY_NAME: &str = "infinito";

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Eq, Debug, Hash)]
pub enum BasicNumber {
  Int(bool, BigUInt),
  Float(bool, BigUFloat),
}
impl BasicNumber {
  pub fn is_zero(&self) -> bool {
    match self {
      Self::Int(_, x) => x.is_zero(),
      Self::Float(_, y_int) => y_int.is_zero(),
    }
  }
  pub fn floor(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x) => {
        let int = x.trunc();
        if !x.has_decimals() {
          return Self::Int(*x_neg, int.clone());
        }
        if *x_neg {
          return Self::Int(*x_neg, &int - &BigUInt::from(1u8));
        }
        Self::Int(*x_neg, int.clone())
      }
    }
  }
  pub fn ceil(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x) => {
        let int = x.trunc();
        if !x.has_decimals() {
          return Self::Int(*x_neg, int.clone());
        }
        if !x_neg {
          return Self::Int(*x_neg, &int + &BigUInt::from(1u8));
        }
        Self::Int(*x_neg, int.clone())
      }
    }
  }
  pub fn round(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x) => {
        let int = x.trunc();
        if !x.has_decimals() {
          return Self::Int(*x_neg, int.clone());
        }
        match x.cmp_decimals_half() {
          std::cmp::Ordering::Greater => Self::Int(*x_neg, &int + &1u8.into()),
          std::cmp::Ordering::Less => Self::Int(*x_neg, &int - &1u8.into()),
          // Si es .5 se redondea al par mas cercano
          std::cmp::Ordering::Equal => Self::Int(
            *x_neg,
            &int + &(int.unit() & 1).into(),
          ),
        }
      }
    }
  }
  pub fn trunc(&self) -> Self {
    match self {
      Self::Int(x_neg, x) => Self::Int(*x_neg, x.clone()),
      Self::Float(x_neg, x) => Self::Int(*x_neg, x.trunc().clone()),
    }
  }
  pub fn is_int(&self) -> bool {
    match self {
      Self::Int(_, _) => true,
      Self::Float(_, x) => !x.has_decimals(),
    }
  }
  pub fn is_negative(&self) -> bool {
    match self {
      Self::Int(x_neg, _) => *x_neg,
      Self::Float(x_neg, _) => *x_neg,
    }
  }
}
impl Display for BasicNumber {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Int(x_neg, x) => write!(f, "{}{}", if *x_neg { "-" } else { "" }, x),
      Self::Float(x_neg, x) => write!(f, "{}{}", if *x_neg { "-" } else { "" }, x),
    }
  }
}
impl PartialEq for BasicNumber {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Int(x_neg, x), Self::Int(y_neg, y)) => x_neg == y_neg && x == y,
      (Self::Float(x_neg, x), Self::Float(y_neg, y)) => x_neg == y_neg && x == y,
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
          return BasicNumber::Int(false, BigUInt::from(0u8));
        }
        if y > x {
          return BasicNumber::Int(*y_neg, y - x);
        }
        BasicNumber::Int(*x_neg, x - y)
      }
      (BasicNumber::Float(x_neg, x), BasicNumber::Float(y_neg, y)) => {
        let neg = if x > y {
          *x_neg
        } else if x < y {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { x - y } else { x + y };
        BasicNumber::Float(neg, value)
      }
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y)) => {
        let neg = if y < x {
          *x_neg
        } else if y > x {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { y - x } else { y + x };
        BasicNumber::Float(neg, value)
      }
      (BasicNumber::Float(x_neg, x), BasicNumber::Int(y_neg, y)) => {
        let neg = if x > y {
          *x_neg
        } else if x < y {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { x - y } else { x + y };
        BasicNumber::Float(neg, value)
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
          return BasicNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          BasicNumber::Int(*x_neg, x - y)
        } else {
          BasicNumber::Int(!y_neg, y - x)
        }
      }

      (BasicNumber::Float(x_neg, x), BasicNumber::Float(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return BasicNumber::Float(*x_neg, x + y);
        }
        if x == y {
          return BasicNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          BasicNumber::Float(*x_neg, x - y)
        } else {
          BasicNumber::Float(!y_neg, y - x)
        }
      }

      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return BasicNumber::Float(*x_neg, y + x);
        }
        if y == x {
          return BasicNumber::Int(false, BigUInt::from(0u8));
        }
        if y < x {
          BasicNumber::Float(*x_neg, y - x)
        } else {
          BasicNumber::Float(!y_neg, y - x)
        }
      }
      (BasicNumber::Float(x_neg, x), BasicNumber::Int(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return BasicNumber::Float(*x_neg, x + y);
        }
        if x == y {
          return BasicNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          BasicNumber::Float(*x_neg, x - y)
        } else {
          BasicNumber::Float(!y_neg, x - y)
        }
      }
    }
  }
}
impl Mul for &BasicNumber {
  type Output = BasicNumber;
  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => BasicNumber::Int(x_neg ^ y_neg, x * y),
      (BasicNumber::Float(x_neg, x), BasicNumber::Float(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, x * y),
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, y * x),
      (BasicNumber::Float(x_neg, x), BasicNumber::Int(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, x * y),
    }
  }
}
impl Div for &BasicNumber {
  type Output = BasicNumber;
  fn div(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (BasicNumber::Int(x_neg, x), BasicNumber::Int(y_neg, y)) => BasicNumber::Int(x_neg ^ y_neg, x / y),
      (BasicNumber::Float(x_neg, x), BasicNumber::Float(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, x / y),
      (BasicNumber::Int(x_neg, x), BasicNumber::Float(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, y / x),
      (BasicNumber::Float(x_neg, x), BasicNumber::Int(y_neg, y)) => BasicNumber::Float(x_neg ^ y_neg, x / y),
    }
  }
}
impl Neg for BasicNumber {
  type Output = BasicNumber;
  fn neg(self) -> Self::Output {
    match self {
      BasicNumber::Int(x_neg, x) => BasicNumber::Int(!x_neg, x),
      BasicNumber::Float(x_neg, x) => BasicNumber::Float(!x_neg, x),
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
      (Self::Float(x_neg, x), Self::Float(y_neg, y)) => {
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
      (Self::Int(x_neg, x), Self::Float(y_neg, y)) => {
                if *x_neg && !*y_neg {
          return std::cmp::Ordering::Less;
        } else if !*x_neg && *y_neg {
          return std::cmp::Ordering::Greater;
        }

        let cmp = y.partial_cmp(x).unwrap();
        if *x_neg && *y_neg {
          return cmp;
        }
        cmp.reverse()
      },
      (Self::Float(x_neg, x), Self::Int(y_neg, y)) => {
                if *x_neg && !*y_neg {
          return std::cmp::Ordering::Less;
        } else if !*x_neg && *y_neg {
          return std::cmp::Ordering::Greater;
        }

        let cmp = x.partial_cmp(y).unwrap();
        if *x_neg && *y_neg {
          return cmp.reverse();
        }
        cmp
      },
    }
  }
}
impl From<String> for BasicNumber {
  fn from(value: String) -> Self {
    Self::Float(false, value.into())
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
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(0u8)));
        }
        if e.is_int() {
          if let BasicNumber::Int(_, e) = e {
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
      (Self::Basic(x), Self::Basic(e)) => {
        if x.is_zero() && e.is_negative() {
          return Self::Infinity;
        }
        if x.is_zero() && e.is_zero() {
          return Self::NaN;
        }
        if x.is_zero() {
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(0u8)));
        }
        if e.is_zero() {
          return Self::Basic(BasicNumber::Int(false, BigUInt::from(1u8)));
        }
        if !e.is_int() {
          return Self::NaN;
        }
        if let BasicNumber::Int(e_neg, e) = e {
          let mut result = BasicNumber::Int(false, BigUInt::from(1u8));
          let mut base = x.clone();
          let mut exponent = e;
          while !exponent.is_zero() {
            if exponent.unit() % 2 == 1 {
              result = &result * &base;
            }
            base = &base * &base;
            exponent = &exponent / &BigUInt::from(2u8);
          }
          if e_neg {
            return Self::Basic(BasicNumber::Float(false, BigUFloat::default()));
          }
          return Self::Basic(result);
        }
        Self::NaN
      }
      (_, _) => Self::NaN,
    }
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
          BasicNumber::Int(false, BigUInt::from(0u8)),
          number,
        ))
      } else if i_exp == 2 {
        Ok(Self::Basic(-number))
      } else if i_exp == 3 {
        Ok(Self::Complex(
          BasicNumber::Int(false, BigUInt::from(0u8)),
          -number,
        ))
      } else {
        Ok(Self::Basic(number))
      };
    }
    if s.contains(".") {
      return Ok(Self::Basic(BasicNumber::Float(false, s.parse()?)));
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
      return Ok(Self::Basic(BasicNumber::Int(false, s.parse()?)));
    }
    Err(format!("No se puede convertir el string '{s}' a un número",))
  }
}
impl<T> From<T> for Number where T: traits::ToDigits {
  fn from(value: T) -> Self {
    Self::Basic(BasicNumber::Int(false, value.into()))
  }
}
impl From<i32> for Number {
  fn from(value: i32) -> Self {
    Self::Basic(BasicNumber::Int(value.is_negative(), value.unsigned_abs().into()))
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
      (Self::Complex(a, b), Self::Complex(c, d)) => {
        Self::Complex(&(&a * &c) - &(&b * &d), &(&a * &d) + &(&c * &b))
      }
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
      (_, Self::Infinity) => Self::Basic(BasicNumber::Int(false, BigUInt::from(0u8))),
      (Self::NegativeInfinity, _) => Self::NegativeInfinity,
      (_, Self::NegativeInfinity) => Self::Basic(BasicNumber::Int(false, BigUInt::from(0u8))),
      (Self::Basic(x), Self::Basic(y)) => Self::Basic(&x / &y),
      (Self::Complex(ref a, ref b), Self::Complex(ref c, ref d)) => {
        let conj = &(&(c * c) + &(d * d));

        Self::Complex(&(&(a * c) + &(b * d)) / conj, &(&(b * c) - &(a * d)) / conj)
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
        Self::Complex(x.clone(), BasicNumber::Int(false, BigUInt::from(0u8)))
          .cmp(&Self::Complex(a.clone(), b.clone()))
      }
      (Self::Complex(a, b), Self::Basic(x)) => Self::Complex(a.clone(), b.clone()).cmp(
        &Self::Complex(x.clone(), BasicNumber::Int(false, BigUInt::from(0u8))),
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
