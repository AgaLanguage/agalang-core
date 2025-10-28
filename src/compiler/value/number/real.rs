use std::borrow::Cow;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use super::binary::Big256 as BigUInt;
use super::float::BigUDecimal as BigUFloat;
use super::traits::Constants as _;

#[derive(Clone, Eq, Debug)]
pub enum RealNumber {
  Int(bool, BigUInt),
  Float(bool, BigUFloat),
}
impl RealNumber {
  pub fn to_float<'a>(&'a self) -> Cow<'a, BigUFloat> {
    match self {
      Self::Float(_, float) => Cow::Borrowed(float),
      Self::Int(_, int) => Cow::Owned(BigUFloat::from(int)),
    }
  }
  pub fn atan2(&self, x: &Self) -> Self {
    let zero = Self::default();
    let pi = Self::pi();

    if x.is_zero() {
      if self.is_zero() {
        panic!("atan2(0,0) is undefined");
      }
      let mut r = pi.div(&Self::from(2u8));
      if self.is_negative() {
        r = -r;
      }
      return r;
    }

    let mut atan_val = self.div(x).arctan(); // implementa arctan con series

    if x.lt(&zero) {
      if self.ge(&zero) {
        atan_val = atan_val.add(&pi); // x<0, y>=0
      } else {
        atan_val = atan_val.sub(&pi); // x<0, y<0
      }
    }

    atan_val
  }
  /// Arctan usando serie de Taylor
  pub fn arctan(&self) -> Self {
    let one = &Self::from(1u8);
    let two = &Self::from(2u8);
    let pi = &Self::pi();

    // --- Caso negativo ---
    if self.is_negative() {
      return -self.clone().neg().arctan();
    }

    // --- Caso x > 1 ---
    if self > one {
      return &(pi / two) - &(one / self).arctan();
    }

    // --- Reducción de rango para x > 0.5 ---
    let half = one / two;
    let (y, offset) = if self > &half {
      // Fórmula: atan(x) = π/4 + atan((x - 1) / (x + 1))
      (&(self - one) / &(self + one), pi / &Self::from(4u8))
    } else {
      (self.clone(), Self::default())
    };

    // --- Serie de Taylor ---
    let mut term = y.clone();
    let mut result = y.clone();
    let y2 = &y * &y;
    let epsilon = Self::epsilon();

    let mut n = Self::from(3u8);
    loop {
      term = &-term * &y2;
      let delta = &term / &n;
      result = &result + &delta;

      if delta.abs() < epsilon {
        break;
      }

      n = &n + two;
    }

    &result + &offset
  }

  /// Calcula e^x con alta precisión usando serie de Taylor
  pub fn exp(&self) -> Self {
    // Casos especiales
    if self.is_zero() {
      return Self::Int(false, BigUInt::from(1u8));
    }

    let one = Self::Int(false, BigUInt::from(1u8));
    let eps = Self::epsilon(); // precisión objetivo

    let mut term = one.clone(); // x^n / n!
    let mut sum = one.clone(); // resultado acumulado
    let mut n = 1u32;

    loop {
      // term *= x / n
      term = term.mul(self).div(&Self::Int(false, BigUInt::from(n)));

      // detener si el término es menor al umbral de precisión
      if term.abs().lt(&eps) {
        break;
      }

      sum = sum.add(&term);
      n += 1;

      // límite de seguridad (previene loops infinitos)
      if n > 4096 {
        eprintln!("Warning: exp() reached iteration limit");
        break;
      }
    }

    sum
  }
  /// Calcula el logaritmo natural (ln) del número real.
  pub fn ln(&self) -> Self {
    match self {
      // ln(0) = -∞
      n if n.is_zero() => panic!("ln(0) → -∞"),
      // ln(1) = 0
      n if n.is_one() => Self::default(),

      // enteros positivos
      Self::Int(false, n) => {
        let val = BigUFloat::new(n.clone(), 0);
        Self::Float(false, val).ln()
      }

      // flotantes positivos
      Self::Float(false, val) => {
        // Si x < 1, usa ln(x) = -ln(1/x)
        if val.lt_one() {
          let inv = &BigUFloat::from(1.0) / val;
          return Self::Float(false, inv).ln().neg();
        }

        // Estimación inicial basada en la magnitud (10^exp ≈ x)
        let mut y = &val.get_exponent().into() * &Self::ln10();

        let val = Self::Float(false, val.clone());
        let one = Self::Int(false, BigUInt::from(1u8));
        let tol = Self::Float(false, BigUFloat::from(1e-80));

        // Iteración Newton-Raphson: y_{n+1} = y - (e^y - x) / e^y
        for _ in 0..128 {
          let e_y = y.exp();
          let delta = one.sub(&val.div(&e_y));
          y = y.sub(&delta);

          if delta.abs().lt(&tol) {
            break;
          }
        }

        y
      }

      // Negativos → resultado complejo (no implementado)
      _ => panic!("ln(x) indefinido para x ≤ 0 (resultado complejo)"),
    }
  }
  /// Calcula la hipotenusa (magnitud) √(a² + b²)
  pub fn hypot(&self, other: &Self) -> Self {
    Self::Float(false, {
      let a: &BigUFloat = &self.to_float();
      let b: &BigUFloat = &other.to_float();

      let a2 = a * a;
      let b2 = b * b;

      a2.add(&b2).sqrt()
    })
  }
  pub fn abs(&self) -> Self {
    match self {
      Self::Int(_, x) => Self::Int(false, x.clone()),
      Self::Float(_, y_int) => Self::Float(false, y_int.clone()),
    }
  }
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
  pub fn is_one(&self) -> bool {
    match self {
      Self::Int(neg, x) => !*neg && x.is_one(),
      Self::Float(neg, y_int) => !*neg && y_int.is_one(),
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
impl Hash for RealNumber {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Int(neg, val) => {
        0u8.hash(state);
        neg.hash(state);
        val.hash(state);
      }
      Self::Float(neg, val) => {
        1u8.hash(state);
        neg.hash(state);
        val.hash(state);
      }
    }
  }
}

impl super::traits::Constants for RealNumber {
  type Base = BigUFloat;
}
impl From<BigUFloat> for RealNumber {
  fn from(value: BigUFloat) -> Self {
    Self::Float(false, value)
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
    let neg = self.is_negative() ^ rhs.is_negative();
    let val = self.to_float().as_ref() / rhs.to_float().as_ref();
    RealNumber::Float(neg, val).into_normalize()
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
    let s = s.trim();
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
    Ok(RealNumber::Float(is_negative, value.parse()?).into_normalize())
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
impl super::traits::Trigonometry for RealNumber {
  fn sin(&self) -> Self {
    // Normalización del ángulo: reduce x dentro de [-π, π]
    let pi = Self::pi();
    let two_pi = Self::tau();
    let mut x = self % &two_pi;
    if x.gt(&pi) {
      x = x.sub(&two_pi);
    }

    let mut term = x.clone(); // primer término: x
    let mut result = term.clone();
    let mut n = 1u64;

    loop {
      // term *= -x² / ((2n)*(2n+1))
      let x2 = x.mul(&x);
      let denom = Self::from((2 * n) * (2 * n + 1));
      term = term.mul(&x2.neg()).div(&denom);
      result = result.add(&term);

      if term.abs().lt(&Self::epsilon()) {
        break;
      }

      n += 1;
    }

    result
  }

  fn cos(&self) -> Self {
    // Normalización del ángulo: reduce x dentro de [-π, π]
    let pi = Self::pi();
    let two_pi = Self::tau();
    let mut x = self.clone().rem(&two_pi);
    if x.gt(&pi) {
      x = x.sub(&two_pi);
    }

    let mut term = Self::from(1u8);
    let mut result = term.clone();
    let mut n = 1u64;

    loop {
      // term *= -x² / ((2n-1)*(2n))
      let x2 = x.mul(&x);
      let denom = Self::from((2 * n - 1) * (2 * n));
      term = term.mul(&x2.neg()).div(&denom);
      result = result.add(&term);

      if term.abs().lt(&Self::epsilon()) {
        break;
      }

      n += 1;
    }

    result
  }
  fn tan(&self) -> Self {
    &self.sin() / &self.cos()
  }
}
impl super::traits::Pow for &RealNumber {
  type Output = RealNumber;
  fn pow(self, rhs: Self) -> Self::Output {
    self.pow_safe(rhs).unwrap()
  }
  fn pow_safe(self, rhs: Self) -> Option<Self::Output> {
    if self.is_zero() {
      if rhs.is_zero() {
        None?;
      }
      return Default::default();
    }

    // Caso especial: x^0 = 1
    if rhs.is_zero() {
      return Some(RealNumber::from(1u8));
    }

    // General: x^y = exp(y * ln(x))
    let ln_x = self.ln();
    let y_ln_x = rhs.mul(&ln_x);
    Some(y_ln_x.exp())
  }
}
impl<T> From<T> for RealNumber
where
  T: super::traits::ToDigits,
{
  fn from(value: T) -> Self {
    Self::Int(false, value.into())
  }
}
impl From<f64> for RealNumber {
  fn from(value: f64) -> Self {
    Self::Float(value.is_sign_negative(), value.abs().into())
  }
}
impl From<i8> for RealNumber {
  fn from(value: i8) -> Self {
    RealNumber::Int(value.is_negative(), value.unsigned_abs().into())
  }
}

#[cfg(test)]
mod tests {
  use crate::compiler::traits::{Pow as _, Trigonometry as _};

  use super::*;

  // Comparación aproximada
  fn approx_eq(a: &RealNumber, b: &RealNumber, tol: f64) -> bool {
    let diff = (a.sub(b)).abs();
    println!("{a}\n{b}\n");
    diff < RealNumber::from(tol)
  }

  // --- ARCTAN ---

  #[test]
  fn test_arctan_basic_values() {
    let zero = RealNumber::from(0.0);
    let one = RealNumber::from(1.0);

    let atan0 = zero.arctan();
    let atan1 = one.arctan();

    assert!(approx_eq(&atan0, &RealNumber::from(0.0), 1e-20));
    // atan(1) = π/4
    let pi = RealNumber::pi();
    let pi_over_4 = pi.div(&RealNumber::from(4u8));
    assert!(approx_eq(&atan1, &pi_over_4, 1e-15));
  }

  #[test]
  fn test_arctan_negative_and_large() {
    let neg_one = RealNumber::from(-1.0);
    let atan_neg1 = neg_one.arctan();
    let pi = RealNumber::pi();
    let pi_over_4 = pi.div(&RealNumber::from(4u8));
    assert!(approx_eq(&atan_neg1, &pi_over_4.neg(), 1e-255));

    // atan(10) ≈ 1.4711276743 (≈ π/2 - atan(0.1))
    let ten = RealNumber::from(10.0);
    let atan10 = ten.arctan();
    let expected = "1.471127674303734591852875571761730851855306377183238262471963519343880455695553844893404788236772162411515656847813754353978995238212134203072377631978956655893898827937824051553659510535022596710919843933276664239361549950957670584150625425647342719081338".parse().unwrap();
    assert!(approx_eq(&atan10, &expected, 1e-253));
  }

  // --- ATAN2 ---

  #[test]
  fn test_atan2_quadrants() {
    let zero = RealNumber::from(0.0);
    let one = RealNumber::from(1.0);
    let neg_one = RealNumber::from(-1.0);
    let pi = RealNumber::pi();

    // (y, x) = (1, 1) → π/4
    let q1 = one.atan2(&one);
    assert!(approx_eq(&q1, &pi.div(&RealNumber::from(4u8)), 1e-15));

    // (1, -1) → 3π/4
    let q2 = one.atan2(&neg_one);
    assert!(approx_eq(
      &q2,
      &pi.mul(&RealNumber::from(3u8)).div(&RealNumber::from(4u8)),
      1e-15
    ));

    // (-1, -1) → -3π/4
    let q3 = neg_one.atan2(&neg_one);
    assert!(approx_eq(
      &q3,
      &pi.mul(&RealNumber::from(-3i8)).div(&RealNumber::from(4u8)),
      1e-15
    ));

    // (-1, 1) → -π/4
    let q4 = neg_one.atan2(&one);
    assert!(approx_eq(
      &q4,
      &pi.clone().neg().div(&RealNumber::from(4u8)),
      1e-15
    ));

    // (1, 0) → π/2
    let qy = one.atan2(&zero);
    assert!(approx_eq(&qy, &pi.div(&RealNumber::from(2u8)), 1e-15));

    // (-1, 0) → -π/2
    let qn = neg_one.atan2(&zero);
    assert!(approx_eq(&qn, &pi.neg().div(&RealNumber::from(2u8)), 1e-15));
  }

  #[test]
  #[should_panic(expected = "atan2(0,0) is undefined")]
  fn test_atan2_zero_zero_panics() {
    let zero = RealNumber::from(0.0);
    let _ = zero.atan2(&zero);
  }

  // --- HYPOT ---

  #[test]
  fn test_hypot_basic() {
    let a = RealNumber::from(3.0);
    let b = RealNumber::from(4.0);
    let h = a.hypot(&b);
    let expected = RealNumber::from(5.0);
    assert!(approx_eq(&h, &expected, 1e-15));
  }

  // --- SIN / COS / TAN ---

  #[test]
  fn test_sin_cos_tan_relations() {
    let pi = RealNumber::pi();

    // sin(π/2) = 1
    let sin_half_pi = pi.div(&RealNumber::from(2u8)).sin();
    assert!(approx_eq(&sin_half_pi, &RealNumber::from(1.0), 1e-15));

    // cos(π/2) = 0
    let cos_half_pi = pi.div(&RealNumber::from(2u8)).cos();
    assert!(approx_eq(&cos_half_pi, &RealNumber::from(0.0), 1e-12));

    // tan(π/4) = 1
    let tan_pi_4 = pi.div(&RealNumber::from(4u8)).tan();
    assert!(approx_eq(&tan_pi_4, &RealNumber::from(1.0), 1e-12));

    // sin²(x) + cos²(x) ≈ 1
    let angle = RealNumber::from(1.0);
    let s = angle.sin();
    let c = angle.cos();
    let identity = s.mul(&s).add(&c.mul(&c));
    assert!(approx_eq(&identity, &RealNumber::from(1.0), 1e-12));
  }

  // --- EXP / LN / POW coherence ---

  #[test]
  fn test_exp_ln_inverse() {
    let nums = [0.5, 1.0, 2.0, 10.0];
    for &n in &nums {
      let x = RealNumber::from(n);
      let ln_x = x.ln();
      let exp_ln_x = ln_x.exp();
      assert!(approx_eq(&x, &exp_ln_x, 1e-12), "exp(ln({n})) != {n}");
    }
  }

  #[test]
  fn test_pow_functionality() {
    let two = RealNumber::from(2.0);
    let three = RealNumber::from(3.0);
    let pow_val = (&two).pow(&three);
    assert!(approx_eq(&pow_val, &RealNumber::from(8.0), 1e-15));

    // raíz cuadrada como exponente fraccionario
    let nine = RealNumber::from(9.0);
    let half = RealNumber::from(0.5);
    let sqrt9 = (&nine).pow(&half);
    assert!(approx_eq(&sqrt9, &RealNumber::from(3.0), 1e-12));
  }
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

    let f = RealNumber::Float(false, BigUFloat::from(std::f64::consts::PI));
    let s_f = format!("{}", f);
    assert_eq!(s_f, "3141592653589793E-15");
  }
}
