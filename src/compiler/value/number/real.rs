use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use super::binary::Big256 as BigUInt;
use super::float::BigUDecimal as BigUFloat;

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Eq, Debug, Hash)]
pub enum RealNumber {
  Int(bool, BigUInt),
  Float(bool, BigUFloat),
}
impl RealNumber {
  pub fn normalize(&mut self) {
    if self.is_zero() {
      *self = Self::Int(false, BigUInt::from(0u8));
      return;
    }
    match self {
      Self::Int(_, x) => x.normalize(),
      Self::Float(neg, x) => {
        if x.has_decimals() {
          // Solo onrmalizamos
          x.normalize();
        } else {
          // Es un entero
          *self = Self::Int(*neg, x.trunc())
        }
      }
    };
  }
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
          std::cmp::Ordering::Equal => Self::Int(*x_neg, &int + &(int.unit() & 1).into()),
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
  pub const fn is_negative(&self) -> bool {
    match self {
      Self::Int(x_neg, _) => *x_neg,
      Self::Float(x_neg, _) => *x_neg,
    }
  }
  fn into_normalize(mut self) -> Self {
    self.normalize();
    self
  }
}
impl Display for RealNumber {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Int(x_neg, x) => write!(f, "{}{}", if *x_neg { "-" } else { "" }, x),
      Self::Float(x_neg, x) => write!(f, "{}{}", if *x_neg { "-" } else { "" }, x),
    }
  }
}
impl PartialEq for RealNumber {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == std::cmp::Ordering::Equal
  }
}
impl Default for RealNumber {
  fn default() -> Self {
    RealNumber::Int(false, BigUInt::from(0u8))
  }
}

impl Add for &RealNumber {
  type Output = RealNumber;
  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (RealNumber::Int(x_neg, x), RealNumber::Int(y_neg, y)) => {
        if x_neg == y_neg {
          return RealNumber::Int(*x_neg, x + y);
        }
        if (x.is_zero() && y.is_zero()) || (x == y) {
          return RealNumber::Int(false, BigUInt::from(0u8));
        }
        if y > x {
          return RealNumber::Int(*y_neg, y - x);
        }
        RealNumber::Int(*x_neg, x - y)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Float(y_neg, y)) => {
        let neg = if x > y {
          *x_neg
        } else if x < y {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { x - y } else { x + y };
        RealNumber::Float(neg, value)
      }
      (RealNumber::Int(x_neg, x), RealNumber::Float(y_neg, y)) => {
        let neg = if y < x {
          *x_neg
        } else if y > x {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { y - x } else { y + x };
        RealNumber::Float(neg, value)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Int(y_neg, y)) => {
        let neg = if x > y {
          *x_neg
        } else if x < y {
          *y_neg
        } else {
          false
        };
        let value = if x_neg != y_neg { x - y } else { x + y };
        RealNumber::Float(neg, value)
      }
    }
    .into_normalize()
  }
}
impl Sub for &RealNumber {
  type Output = RealNumber;

  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (RealNumber::Int(x_neg, x), RealNumber::Int(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return RealNumber::Int(*x_neg, x + y);
        }
        if x == y {
          return RealNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          RealNumber::Int(*x_neg, x - y)
        } else {
          RealNumber::Int(!y_neg, y - x)
        }
      }

      (RealNumber::Float(x_neg, x), RealNumber::Float(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return RealNumber::Float(*x_neg, x + y);
        }
        if x == y {
          return RealNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          RealNumber::Float(*x_neg, x - y)
        } else {
          RealNumber::Float(!y_neg, y - x)
        }
      }

      (RealNumber::Int(x_neg, x), RealNumber::Float(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return RealNumber::Float(*x_neg, y + x);
        }
        if y == x {
          return RealNumber::Int(false, BigUInt::from(0u8));
        }
        if y < x {
          RealNumber::Float(*x_neg, y - x)
        } else {
          RealNumber::Float(!y_neg, y - x)
        }
      }
      (RealNumber::Float(x_neg, x), RealNumber::Int(y_neg, y)) => {
        if x_neg != y_neg {
          // x - (-y) = x + y  || -x - y = -(x + y)
          return RealNumber::Float(*x_neg, x + y);
        }
        if x == y {
          return RealNumber::Int(false, BigUInt::from(0u8));
        }
        if x > y {
          RealNumber::Float(*x_neg, x - y)
        } else {
          RealNumber::Float(!y_neg, x - y)
        }
      }
    }
    .into_normalize()
  }
}
impl Mul for &RealNumber {
  type Output = RealNumber;
  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (RealNumber::Int(x_neg, x), RealNumber::Int(y_neg, y)) => {
        RealNumber::Int(x_neg ^ y_neg, x * y)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Float(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, x * y)
      }
      (RealNumber::Int(x_neg, x), RealNumber::Float(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, y * x)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Int(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, x * y)
      }
    }
    .into_normalize()
  }
}
impl Div for &RealNumber {
  type Output = RealNumber;
  fn div(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (RealNumber::Int(x_neg, x), RealNumber::Int(y_neg, y)) => {
        RealNumber::Int(x_neg ^ y_neg, x / y)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Float(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, x / y)
      }
      (RealNumber::Int(x_neg, x), RealNumber::Float(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, y / x)
      }
      (RealNumber::Float(x_neg, x), RealNumber::Int(y_neg, y)) => {
        RealNumber::Float(x_neg ^ y_neg, x / y)
      }
    }
    .into_normalize()
  }
}
impl Neg for RealNumber {
  type Output = RealNumber;
  fn neg(self) -> Self::Output {
    match self {
      RealNumber::Int(x_neg, x) => {
        let new_neg = if x.is_zero() { false } else { !x_neg };
        RealNumber::Int(new_neg, x)
      }
      RealNumber::Float(x_neg, x) => {
        let new_neg = if x.is_zero() { false } else { !x_neg };
        RealNumber::Float(new_neg, x)
      }
    }
    .into_normalize()
  }
}
impl Rem for &RealNumber {
  type Output = RealNumber;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = (self / rhs).trunc();
    let mul = rhs * &div;
    (self - &mul).into_normalize()
  }
}
impl PartialOrd for RealNumber {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for RealNumber {
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
      }
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
      }
    }
  }
}

impl std::str::FromStr for RealNumber {
  type Err = super::NumberError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use super::traits::FromStrRadix;
    Self::from_str_radix(s, 10)
  }
}
impl super::traits::FromStrRadix for RealNumber {
  fn from_str_radix(src: &str, radix: u8) -> Result<Self, super::NumberError> {
    if !(2..=36).contains(&radix) {
      return Err(super::NumberError::Radix(radix));
    }
    let s = src.trim();
    if s.is_empty() {
      return Err(super::NumberError::Empty);
    }
    let (is_negative, value) = if s.starts_with("-") {
      (true, s.trim_start_matches("-"))
    } else if s.starts_with("+") {
      (false, s.trim_start_matches("+"))
    } else {
      (false, s)
    };
    Ok(Self::Int(
      is_negative,
      BigUInt::from_str_radix(value, radix)?,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_creation_int_float() {
    let int_val = RealNumber::Int(false, BigUInt::from(42u8));
    let float_val = RealNumber::Float(false, "3.14".parse().unwrap());

    assert!(!int_val.is_negative());
    assert!(!float_val.is_negative());
    assert!(!int_val.is_zero());
    assert!(!float_val.is_zero());
    assert!(int_val.is_int());
    assert!(!float_val.is_int());
  }

  #[test]
  fn test_addition() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let b = RealNumber::Int(false, BigUInt::from(5u8));
    let result = &a + &b;
    assert_eq!(result, RealNumber::Int(false, BigUInt::from(15u8)));
    let f1 = RealNumber::Float(false, BigUFloat::from(2.5));
    let f2 = RealNumber::Float(false, BigUFloat::from(1.5));
    let result_f = &f1 + &f2;
    assert_eq!(result_f, RealNumber::Float(false, BigUFloat::from(4.0)));

    // Mixed Int + Float
    let result_mix = &a + &f1;
    assert_eq!(result_mix, RealNumber::Float(false, BigUFloat::from(12.5)));
  }

  #[test]
  fn test_subtraction() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let b = RealNumber::Int(false, BigUInt::from(5u8));
    let result = &a - &b;
    assert_eq!(result, RealNumber::Int(false, BigUInt::from(5u8)));

    let f1 = RealNumber::Float(false, BigUFloat::from(2.5));
    let f2 = RealNumber::Float(false, BigUFloat::from(1.5));
    let result_f = &f1 - &f2;
    assert_eq!(result_f, RealNumber::Float(false, BigUFloat::from(1.0)));
  }

  #[test]
  fn test_multiplication() {
    let a = RealNumber::Int(false, BigUInt::from(3u8));
    let b = RealNumber::Int(true, BigUInt::from(4u8));
    let result = &a * &b;
    assert_eq!(result, RealNumber::Int(true, BigUInt::from(12u8)));

    let f1 = RealNumber::Float(false, BigUFloat::from(2.0));
    let f2 = RealNumber::Float(true, BigUFloat::from(3.5));
    let result_f = &f1 * &f2;
    assert_eq!(result_f, RealNumber::Float(true, BigUFloat::from(7.0)));
  }

  #[test]
  fn test_division() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let b = RealNumber::Int(false, BigUInt::from(2u8));
    let result = &a / &b;
    assert_eq!(result, RealNumber::Int(false, BigUInt::from(5u8)));

    let f1 = RealNumber::Float(false, BigUFloat::from(7.5));
    let f2 = RealNumber::Float(false, BigUFloat::from(2.5));
    let result_f = &f1 / &f2;
    assert_eq!(result_f, RealNumber::Float(false, BigUFloat::from(3.0)));
  }

  #[test]
  fn test_negation() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let neg_a = -a.clone();
    assert_eq!(neg_a, RealNumber::Int(true, BigUInt::from(10u8)));

    let f = RealNumber::Float(false, BigUFloat::from(2.5));
    let neg_f = -f.clone();
    assert_eq!(neg_f, RealNumber::Float(true, BigUFloat::from(2.5)));
  }

  #[test]
  fn test_floor_ceil_round_trunc() {
    let f = RealNumber::Float(false, BigUFloat::from(3.7));
    assert_eq!(f.floor(), RealNumber::Int(false, BigUInt::from(3u8)));
    assert_eq!(f.ceil(), RealNumber::Int(false, BigUInt::from(4u8)));
    assert_eq!(f.trunc(), RealNumber::Int(false, BigUInt::from(3u8)));

    let f2 = RealNumber::Float(false, BigUFloat::from(2.5));
    let rounded = f2.round();
    // Redondea al par más cercano
    assert_eq!(rounded, RealNumber::Int(false, BigUInt::from(2u8)));
  }

  #[test]
  fn test_remainder() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let b = RealNumber::Int(false, BigUInt::from(3u8));
    let rem = &a % &b;
    assert_eq!(rem, RealNumber::Int(false, BigUInt::from(1u8)));

    let f1 = RealNumber::Float(false, BigUFloat::from(7.6));
    let f2 = RealNumber::Float(false, BigUFloat::from(2.5));
    let rem_f = &f1 % &f2;
    assert_eq!(rem_f, RealNumber::Float(false, BigUFloat::from(0.1)));

    let f3 = RealNumber::Float(false, BigUFloat::from(7.5));
    let f4 = RealNumber::Float(false, BigUFloat::from(2.5));
    let rem_f = &f3 % &f4;
    let mut zero = RealNumber::Float(false, BigUFloat::from(0.0));
    zero.normalize();
    assert_eq!(rem_f, zero);
  }

  #[test]
  fn test_comparisons() {
    let a = RealNumber::Int(false, BigUInt::from(10u8));
    let b = RealNumber::Int(true, BigUInt::from(10u8));
    assert!(a > b);

    let f1 = RealNumber::Float(false, BigUFloat::from(3.5));
    let f2 = RealNumber::Float(false, BigUFloat::from(3.5));
    assert_eq!(f1, f2);

    let f3 = RealNumber::Float(false, BigUFloat::from(10.0));
    assert_eq!(a, f3);
  }

  #[test]
  fn test_display() {
    let a = RealNumber::Int(true, BigUInt::from(42u8));
    let s = format!("{}", a);
    assert_eq!(s, "-42");

    #[deny(clippy::approx_constant)]
    let f = RealNumber::Float(false, BigUFloat::from(std::f64::consts::PI));
    let s_f = format!("{}", f);
    assert_eq!(s_f, "3141592653589793E-15");
  }
}
