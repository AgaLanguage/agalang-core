use std::{
  fmt::Display,
  hash::{Hash, Hasher},
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

use crate::{
  compiler::{
    traits::{BaseConstants as _, Trigonometry as _},
    value::number::traits::{AsNumber as _, FromStrRadix},
  },
  util::{OnError, OnSome},
  StructTag,
};

mod binary;
pub mod traits;
pub use binary::Big256 as BigUInt;
mod float;
pub use float::BigUDecimal as BigUFloat;

mod real;
pub use real::RealNumber;
use traits::Constants;

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

#[derive(Clone, Eq, Default)]
pub enum Number {
  #[default]
  NaN,
  Infinity,
  NegativeInfinity,
  Real(RealNumber),
  Complex(RealNumber, RealNumber),
}
impl Number {
  pub fn abs(&self) -> Self {
    match self {
      Self::NaN => Self::NaN,
      Self::Infinity | Self::NegativeInfinity => Self::Infinity,
      Self::Real(real) => Self::Real(real.abs()),
      Self::Complex(real, imag) => {
        let r2 = real * real;
        let i2 = imag * imag;
        Self::Real(r2.hypot(&i2))
      }
    }
  }
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
  pub fn is_one(&self) -> bool {
    match self {
      Self::NaN | Self::Infinity | Self::NegativeInfinity => false,
      Self::Real(x) => x.is_one(),
      Self::Complex(x, y) => x.is_one() && y.is_zero(),
    }
  }
  pub const fn is_negative(&self) -> bool {
    match self {
      Self::NaN | Self::Infinity => false,
      Self::NegativeInfinity => true,
      Self::Real(x) | Self::Complex(x, _) => x.is_negative(),
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
impl Hash for Number {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Number::NaN => 0u8.hash(state),
      Number::Infinity => 1u8.hash(state),
      Number::NegativeInfinity => 2u8.hash(state),
      Number::Real(r) => {
        3u8.hash(state);
        r.hash(state);
      }
      Number::Complex(re, im) => {
        4u8.hash(state);
        re.hash(state);
        im.hash(state);
      }
    }
  }
}

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
impl traits::Pow for &Number {
  type Output = Number;
  fn pow(self, exp: Self) -> Self::Output {
    use Number::*;
    // TODO: implementar correctamente las potencias. Esta implementacion es muy basica.
    match (self, exp) {
      (NaN, _) | (_, NaN) => NaN,
      (Infinity | NegativeInfinity, Infinity) => Infinity,
      (Infinity | NegativeInfinity, NegativeInfinity) => Real(RealNumber::default()),
      (Infinity | NegativeInfinity, n) => {
        if n.is_zero() {
          return Real(1u8.into());
        }
        if n < &Real(Default::default()) {
          return Real(RealNumber::Int(false, BigUInt::from(0u8)));
        }
        Infinity
      }
      (n, Infinity | NegativeInfinity) => {
        if n.is_zero() {
          return if exp.is_negative() {
            Real(0u8.into())
          } else {
            return Infinity;
          };
        }
        if n < &Real(Default::default()) {
          return Real(RealNumber::Int(false, BigUInt::from(0u8)));
        }
        let abs = n.abs();
        let one = Real(1u8.into());
        match abs.cmp(&one) {
          std::cmp::Ordering::Equal => one,
          std::cmp::Ordering::Greater => {
            if exp.is_negative() {
              Real(0u8.into())
            } else {
              Infinity
            }
          }
          std::cmp::Ordering::Less => {
            if exp.is_negative() {
              Infinity
            } else {
              Real(0u8.into())
            }
          }
        }
      }
      (a, b) if a.is_zero() && b.is_zero() => NaN,
      (Real(a), Real(b)) => {
        let (an, a, bn, b) = match (a, b) {
          // ambos enteros
          (a, RealNumber::Int(bn, bi)) => {
            if a.is_zero() {
              return Real(RealNumber::Int(false, BigUInt::default()));
            }
            if bi.is_zero() {
              return Real(RealNumber::Int(false, BigUInt::from(1u8)));
            }
            let (neg, res) = match a {
              RealNumber::Int(an, ai) => (an, BigUFloat::new(ai.pow(bi), 0u8)),
              RealNumber::Float(an, af) => (an, af.pow(bi)),
            };
            let res = if *bn {
              BigUFloat::from(1.0).div(&res)
            } else {
              res
            };
            return Real(RealNumber::Float(*neg && bi.is_odd(), res));
          }
          // cualquier combinación con floats
          (RealNumber::Float(an, af), RealNumber::Float(bn, bf)) => (an, af, bn, bf),
          (RealNumber::Int(an, ai), RealNumber::Float(bn, bf)) => {
            (an, &BigUFloat::new(ai.clone(), 0), bn, bf)
          }
        };

        let a_pow_b = RealNumber::Float(false, a.clone()).pow(&RealNumber::Float(false, b.clone()));
        // Si base negativa y exponente no entero => resultado complejo
        if *an && b.has_decimals() {
          // (-x)^y = x^y * e^(iπy)
          let pi_b = RealNumber::Float(false, BigUFloat::pi().mul(b));
          let imag = pi_b.sin();
          let real = pi_b.cos();
          return Complex(a_pow_b.mul(&real), a_pow_b.mul(&imag));
        }

        let res = if *bn {
          RealNumber::from(1u8).div(&a_pow_b)
        } else {
          a_pow_b
        };
        let real = if *an && b.is_odd() {
          -res
        }else {res};

        Real(real)
      }
      (Real(a), Complex(br, bi)) => {
        // a^(rb + i ib) = a^rb * e^(i * ib * ln(a))
        if a.is_negative() {
          // para base negativa, extendemos a forma polar compleja
          let theta = &RealNumber::pi(); // arg(-x) = π
          let ln_r = a.abs().ln();
          let (re, im) = (
            ln_r.mul(&br.add(&bi.mul(theta).neg())),
            ln_r.mul(&bi.add(&br.mul(theta))),
          );
          Complex(re.exp(), im.exp())
        } else {
          let ln_r = a.ln();
          let e_pow_re = ln_r.mul(br).exp();
          let imag_part = bi.mul(&ln_r);
          let cos_i = imag_part.cos();
          let sin_i = imag_part.sin();
          Complex(e_pow_re.mul(&cos_i), e_pow_re.mul(&sin_i))
        }
      }
      (Complex(a, b), Real(r)) => {
        // (a + bi)^r = e^{r ln(a+bi)}
        let lnz = complex_ln(a, b);
        let re = lnz.0.mul(r);
        let im = lnz.1.mul(r);
        complex_exp(&re, &im)
        
      }
      (Complex(a, b), Complex(c, d)) => {
        // (a + bi)^(c + di) = e^{(c + di) * ln(a + bi)}
        let lnz = complex_ln(a, b);
        let re = c.mul(&lnz.0).sub(&d.mul(&lnz.1));
        let im = c.mul(&lnz.1).add(&d.mul(&lnz.0));
        complex_exp(&re, &im)
      }
    }
  }
}
impl Constants for Number {
  type Base = BigUFloat;
}
impl From<BigUFloat> for Number {
  fn from(value: BigUFloat) -> Self {
    Self::Real(value.into())
  }
}

impl Add<RealNumber> for &Number {
  type Output = Number;
  fn add(self, rhs: RealNumber) -> Self::Output {
    self + &Number::Real(rhs)
  }
}

// Funciones auxiliares
fn complex_ln(a: &RealNumber, b: &RealNumber) -> (RealNumber, RealNumber) {
  // ln(a + bi) = ln(|z|) + i*atan(b/a)
  let r = a.hypot(b);
  let theta = b.atan2(a);
  (r.ln(), theta)
}

fn complex_exp(re: &RealNumber, im: &RealNumber) -> Number {
  // e^(x + iy) = e^x (cos y + i sin y)
  let ex = re.exp();
  let cos_y = im.cos();
  let sin_y = im.sin();
  Number::Complex(ex.mul(&cos_y), ex.mul(&sin_y))
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
    Ok(Self::Real(RealNumber::from_str(s)?))
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
