use crate::compiler::traits::AsNumber;

use super::traits::BaseConstants;
use super::BigUInt;

use std::{
  cmp::Ordering,
  fmt::Display,
  hash,
  ops::{Add, Div, Mul, Sub},
};

const PI_VAL: [u8; 107] = [
  48, 128, 129, 92, 194, 153, 239, 222, 150, 89, 57, 35, 170, 203, 111, 20, 83, 168, 65, 67, 213,
  228, 122, 88, 140, 194, 200, 12, 77, 115, 12, 178, 166, 121, 44, 210, 213, 11, 243, 122, 81, 117,
  170, 230, 212, 177, 116, 159, 0, 39, 217, 14, 94, 135, 104, 214, 228, 39, 216, 1, 17, 161, 49,
  132, 134, 58, 104, 252, 127, 189, 26, 16, 88, 50, 45, 28, 17, 19, 54, 163, 10, 48, 10, 157, 40,
  245, 187, 171, 222, 140, 185, 126, 130, 164, 236, 163, 240, 96, 15, 91, 93, 7, 219, 173, 128,
  172, 1,
];
const TAU_VAL: [u8; 107] = [
  97, 0, 3, 185, 132, 51, 223, 189, 45, 179, 114, 70, 84, 151, 223, 40, 166, 80, 131, 134, 170,
  201, 245, 176, 24, 133, 145, 25, 154, 230, 24, 100, 77, 243, 88, 164, 171, 23, 230, 245, 162,
  234, 84, 205, 169, 99, 233, 62, 1, 78, 178, 29, 188, 14, 209, 172, 201, 79, 176, 3, 34, 66, 99,
  8, 13, 117, 208, 248, 255, 122, 53, 32, 176, 100, 90, 56, 34, 38, 108, 70, 21, 96, 20, 58, 81,
  234, 119, 87, 189, 25, 115, 253, 4, 73, 217, 71, 225, 193, 30, 182, 186, 14, 182, 91, 1, 89, 3,
];
const EULER_VAL: [u8; 107] = [
  88, 168, 7, 30, 192, 221, 219, 235, 222, 171, 242, 3, 216, 8, 4, 209, 125, 17, 32, 22, 94, 252,
  172, 206, 142, 0, 102, 65, 163, 232, 22, 254, 225, 143, 143, 4, 81, 50, 160, 108, 1, 142, 181,
  167, 126, 227, 169, 134, 58, 54, 219, 241, 194, 222, 18, 115, 64, 101, 218, 213, 148, 198, 167,
  145, 103, 111, 94, 238, 106, 135, 154, 221, 94, 145, 196, 159, 184, 105, 242, 218, 117, 185, 26,
  6, 35, 100, 154, 80, 150, 225, 208, 73, 132, 185, 148, 213, 173, 11, 221, 4, 114, 9, 33, 181,
  195, 114, 1,
];
const LN10_VAL: [u8; 107] = [
  228, 239, 230, 230, 215, 236, 72, 204, 148, 199, 240, 182, 208, 113, 233, 213, 78, 255, 18, 39,
  145, 140, 40, 74, 81, 113, 5, 55, 134, 103, 111, 113, 11, 97, 211, 86, 41, 78, 168, 39, 72, 68,
  108, 94, 42, 205, 114, 252, 200, 131, 152, 206, 212, 148, 156, 218, 200, 140, 244, 138, 254, 90,
  3, 97, 153, 132, 203, 91, 92, 246, 29, 75, 6, 10, 208, 207, 166, 216, 219, 78, 52, 76, 178, 27,
  145, 8, 53, 248, 254, 31, 222, 13, 232, 38, 147, 41, 116, 197, 229, 138, 162, 202, 232, 153, 16,
  58, 1,
];

#[derive(Clone, Debug, Default)]
/// Representa una estructura decimal
/// Presicion de 255 digitos
/// Representa un número decimal como `mantissa * 10^(-exponent)`.
/// Ejemplo: `mantissa = 1234`, `exponent = 2` ⇒ 12.34
pub struct BigUDecimal {
  /// parte significativa
  mantissa: BigUInt,
  /// desplazamiento decimal (exponente negativo)
  exponent: u8,
}

impl BigUDecimal {
  pub fn len(&self) -> usize {
    self.mantissa.len()
  }
  pub fn get_exponent(&self) -> u8 {
    self.exponent
  }
  pub fn new(mantissa: BigUInt, exponent: u8) -> Self {
    let mut d = Self { mantissa, exponent };
    d.normalize();
    d
  }
  #[inline]
  pub fn is_zero(&self) -> bool {
    self.mantissa.is_zero()
  }
  #[inline]
  pub fn is_one(&self) -> bool {
    self.mantissa.is_one() && self.exponent == 0
  }
  #[inline]
  pub fn is_odd(&self) -> bool {
    self.mantissa.is_odd() && !self.has_decimals()
  }
  /// Devuelve true si self < EPSILON
  #[inline]
  pub fn is_tiny(&self) -> bool {
    self.lt(&BigUDecimal::epsilon())
  }
  #[inline]
  /// Devuelve true si self < 1
  pub fn lt_one(&self) -> bool {
    self < &Self::from(1u8)
  }
  /// Compara la parte decimal de `self` con X.5
  pub fn cmp_decimals_half(&self) -> Ordering {
    // Se normaliza en la creacion
    if self.exponent == 0 {
      return Ordering::Less; // no hay parte decimal
    }

    // parte fraccionaria = mantissa % 10^exponent
    let pow10 = BigUInt::from_pow10(self.exponent);
    let frac_part = &self.mantissa % &pow10;

    // mitad = 5 * 10^(exponent - 1)
    if self.exponent == 0 {
      return Ordering::Less;
    }
    let half = if self.exponent == 1 {
      BigUInt::from(5u8)
    } else {
      &BigUInt::from(5u8) * &BigUInt::from_pow10(self.exponent - 1)
    };

    frac_part.cmp(&half)
  }
  /// Compara si tiene decimales.
  pub fn has_decimals(&self) -> bool {
    let dec = self.exponent as usize;
    if dec == 0 {
      return false;
    }
    let s = self.mantissa.to_string();
    if dec < s.len() {
      &s[(s.len() - dec)..]
    } else {
      &s
    }
    .chars()
    .any(|c| c != '0')
  }
  /// Elimina los decimales manteniendo solo la parte entera
  pub fn trunc(&self) -> BigUInt {
    let s = self.mantissa.to_string();
    let len = s.len();
    let dec = self.exponent as usize;
    if dec >= len { "0" } else { &s[..(len - dec)] }
      .parse()
      .unwrap()
  }
  pub fn normalize(&mut self) {
    if self.mantissa.is_zero() {
      // Si la mantissa es cero, reseteamos el exponente
      self.exponent = 0;
      return;
    }

    let mut s = self.mantissa.to_string();
    let mut zeros = 0;

    // Contamos ceros al final de la mantissa
    while s.ends_with('0') && zeros < self.exponent {
      zeros += 1;
      s.pop();
    }

    // Ajustamos mantissa y exponente
    self.mantissa = s.parse().unwrap();
    self.exponent -= zeros;
  }
  #[inline]
  pub fn into_normalize(mut self) -> Self {
    self.normalize();
    self
  }
  fn establish(&self, exponent: u8) -> Self {
    let mantissa = self.to_absolute(exponent - self.exponent);
    Self { mantissa, exponent }
  }
  fn to_absolute(&self, exponent: u8) -> BigUInt {
    &self.mantissa * &BigUInt::from_pow10(exponent)
  }
  /// Redondea el número al múltiplo más cercano de epsilon
  fn increase_exponent_with_round(mut self) -> Self {
    if self.exponent < 255 {
      self.exponent += 1;
      return self;
    }
    let (d, u) = {
      let string = self.mantissa.to_string();
      let len = string.len();
      let val = if len >= 2 {
        string
          .get((len - 2)..)
          .map(|f| f.parse::<u8>().unwrap_or_default())
          .unwrap_or_default()
      } else {
        string.parse::<u8>().unwrap_or_default()
      };
      (val / 10, val % 10)
    };
    let ten = BigUInt::from(10u8);
    if u > 5 || (u == 5 && d & 1 == 1) {
      self.mantissa = self.mantissa.add(&ten)
    }
    self.mantissa /= &ten;
    self.into_normalize()
  }
  /// Calcula la raíz cuadrada del número con precisión arbitraria.
  /// Usa el método de Newton-Raphson.
  pub fn sqrt(&self) -> Self {
    // caso trivial
    if self.is_zero() {
      return self.clone();
    }

    // Se utiliza 10² para exponent + 1 y redondear al final
    let mantissa = (self * &Self::from(100u8)).mantissa;

    // Convertir mantissa a un BigUFloat para trabajar en decimal
    let n = Self {
      mantissa,
      exponent: 0,
    };

    // --- Método de Newton-Raphson ---
    // x_{n+1} = 0.5 * (x_n + N / x_n)
    let mut x = n.approx_start(); // estimación inicial
    let two = Self::from(2.0);
    let tol = Self::epsilon();

    loop {
      let prev = x.clone();
      let div = n.div(&x);
      x = x.add(&div).div(&two);

      // Si converge (no cambia en varios dígitos), salimos
      if x.sub(&prev) < tol {
        break;
      }
    }

    // Ajustar exponente con redondeo √25 seria 4.9999... en lugar de 5 de no usarse
    x.increase_exponent_with_round()
  }
  /// Crea una estimación inicial rápida (basada en longitud de mantissa)
  fn approx_start(&self) -> Self {
    let digits = self.mantissa.len();
    // sqrt(10^n) ≈ 10^(n/2)
    let est_exp = digits as u8 / 2;
    Self {
      mantissa: BigUInt::from(10u32.pow(est_exp as u32)),
      exponent: 0,
    }
  }
  pub fn exp(&self) -> Self {
    if self.is_zero() {
      return BigUDecimal::from(1.0);
    }

    // Serie de Taylor: e^x = 1 + x + x²/2! + x³/3! + ...
    let mut term = BigUDecimal::from(1.0);
    let mut result = BigUDecimal::from(1.0);
    let mut n = 1u64;

    loop {
      term = term.mul(self).div(&BigUDecimal::from(n));
      result = result.add(&term);

      // criterio de parada: el término ya no cambia el resultado
      if term.is_tiny() {
        break;
      }
      n += 1;
    }

    result
  }
}

impl std::str::FromStr for BigUDecimal {
  type Err = super::NumberError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.is_empty() {
      return Err(super::NumberError::Empty);
    }
    let parts: Vec<&str> = s.split(".").collect();
    if parts.len() > 2 {
      return Err(super::NumberError::InvalidCharacter('.'));
    }
    let int_part = parts[0];
    let mut dec_part = *parts.get(1).unwrap_or(&"0");
    if dec_part.len() > 255 {
      dec_part = &dec_part[..255]
    }

    let value = format!("{int_part}{dec_part}").as_number()?;
    let exponent = dec_part.len() as u8;

    Ok(BigUDecimal::new(value, exponent))
  }
}
impl From<f64> for BigUDecimal {
  fn from(value: f64) -> Self {
    value.abs().to_string().parse().unwrap()
  }
}

impl Display for BigUDecimal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}E-{}", self.mantissa, self.exponent)
  }
}

enum Refable<'a, T> {
  IsRef(&'a T),
  NotRef(T),
}
impl<'a, T> std::ops::Deref for Refable<'a, T> {
  type Target = T;
  fn deref(&self) -> &T {
    match self {
      Self::IsRef(t) => t,
      Self::NotRef(t) => t,
    }
  }
}
macro_rules! establish {
  ($lhs:expr, $rhs:expr) => {{
    use Refable::*;
    let exponent = $lhs.exponent.max($rhs.exponent);

    // Igualamos exponentes para poder operar
    let lhs = if $lhs.exponent < exponent {
      NotRef($lhs.establish(exponent))
    } else {
      IsRef($lhs)
    };

    let rhs = if $rhs.exponent < exponent {
      NotRef($rhs.establish(exponent))
    } else {
      IsRef($rhs)
    };

    (lhs, rhs)
  }};
}

impl Add for &BigUDecimal {
  type Output = BigUDecimal;

  fn add(self, rhs: Self) -> Self::Output {
    let (lhs, rhs) = establish!(self, rhs);

    let mantissa = &lhs.mantissa + &rhs.mantissa;

    BigUDecimal::new(mantissa, lhs.exponent)
  }
}
impl Sub for &BigUDecimal {
  type Output = BigUDecimal;

  fn sub(self, rhs: Self) -> Self::Output {
    let (lhs, rhs) = establish!(self, rhs);

    let mantissa = &lhs.mantissa - &rhs.mantissa;

    BigUDecimal::new(mantissa, lhs.exponent)
  }
}
impl Mul for &BigUDecimal {
  type Output = BigUDecimal;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut mantissa = &self.mantissa * &rhs.mantissa;

    // Malabares para el exponente
    let pre_exponent: u16 = self.exponent as u16 + rhs.exponent as u16;
    let (exponent, rest) = if pre_exponent > 255 {
      (255u8, (pre_exponent - 255) as u8)
    } else {
      (pre_exponent as u8, 0)
    };

    // Truncamos mantissa si hay exceso de decimales
    if rest > 0 {
      mantissa /= &BigUInt::from_pow10(rest);
    }

    BigUDecimal::new(mantissa, exponent)
  }
}
impl Div for &BigUDecimal {
  type Output = BigUDecimal;

  fn div(self, rhs: Self) -> Self::Output {
    // Expandimos self a 10^255 para preservar precisión
    let mut mantissa = &self.to_absolute(255) / &rhs.mantissa;

    // Malabares para el exponente
    let pre_exponent: u16 = 0xFF + self.exponent as u16 - rhs.exponent as u16;
    let (exponent, rest) = if pre_exponent > 255 {
      (255u8, (pre_exponent - 255) as u8)
    } else {
      (pre_exponent as u8, 0)
    };

    // Truncamos mantissa si hay exceso de decimales
    if rest > 0 {
      mantissa /= &BigUInt::from_pow10(rest);
    }

    BigUDecimal::new(mantissa, exponent)
  }
}

impl Add<&BigUInt> for &BigUDecimal {
  type Output = BigUDecimal;
  fn add(self, rhs: &BigUInt) -> Self::Output {
    self
      + &BigUDecimal {
        mantissa: rhs.clone(),
        exponent: 0,
      }
  }
}
impl Sub<&BigUInt> for &BigUDecimal {
  type Output = BigUDecimal;
  fn sub(self, rhs: &BigUInt) -> Self::Output {
    self
      - &BigUDecimal {
        mantissa: rhs.clone(),
        exponent: 0,
      }
  }
}
impl Mul<&BigUInt> for &BigUDecimal {
  type Output = BigUDecimal;
  fn mul(self, rhs: &BigUInt) -> Self::Output {
    self
      * &BigUDecimal {
        mantissa: rhs.clone(),
        exponent: 0,
      }
  }
}
impl Div<&BigUInt> for &BigUDecimal {
  type Output = BigUDecimal;
  fn div(self, rhs: &BigUInt) -> Self::Output {
    self
      / &BigUDecimal {
        mantissa: rhs.clone(),
        exponent: 0,
      }
  }
}
impl super::traits::Pow<&BigUInt> for &BigUDecimal {
  type Output = BigUDecimal;
  fn pow(self, rhs: &BigUInt) -> Self::Output {
    self.pow_safe(rhs).unwrap()
  }
  fn pow_safe(self, rhs: &BigUInt) -> Option<Self::Output> {
    if self.is_zero() && rhs.is_zero() {
      None?
    }
    // 0^1 = 0, etc.
    if self.is_zero() {
      return Some(BigUDecimal::default());
    }
    // 1^0 = 1, etc.
    if rhs.is_zero() {
      return Some(BigUDecimal::from(1.0));
    }

    let mut result = BigUDecimal::from(1.0);
    let mut b = self.clone();
    let mut e = rhs.clone();

    while !e.is_zero() {
      if e.is_odd() {
        result = &result * &b;
      }
      b = &b * &b;
      e.div2_inplace();
    }

    Some(result)
  }
}

impl super::traits::BaseConstants for BigUDecimal {
  fn pi() -> Self {
    BigUDecimal::new(BigUInt::new(PI_VAL.to_vec()), 255)
  }
  fn tau() -> Self {
    BigUDecimal::new(BigUInt::new(TAU_VAL.to_vec()), 255)
  }
  fn euler() -> Self {
    BigUDecimal::new(BigUInt::new(EULER_VAL.to_vec()), 255)
  }
  fn ln10() -> Self {
    BigUDecimal::new(BigUInt::new(LN10_VAL.to_vec()), 255)
  }
  fn epsilon() -> Self {
    BigUDecimal {
      mantissa: BigUInt::from(1u8),
      exponent: 255,
    }
  }
}

impl PartialEq<BigUInt> for BigUDecimal {
  fn eq(&self, other: &BigUInt) -> bool {
    !self.has_decimals() && &self.trunc() == other
  }
}
impl PartialOrd for BigUDecimal {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
impl PartialOrd<BigUInt> for BigUDecimal {
  fn partial_cmp(&self, other: &BigUInt) -> Option<Ordering> {
    let ord = self.trunc().cmp(other);
    let ord = if ord == Ordering::Equal && self.has_decimals() {
      Ordering::Greater
    } else {
      ord
    };
    Some(ord)
  }
}
impl Ord for BigUDecimal {
  fn cmp(&self, other: &Self) -> Ordering {
    let (lhs, rhs) = establish!(self, other);
    lhs.mantissa.cmp(&rhs.mantissa)
  }
}

impl PartialEq for BigUDecimal {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}
impl Eq for BigUDecimal {}
impl hash::Hash for BigUDecimal {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    self.mantissa.hash(state);
    self.exponent.hash(state);
  }
}
impl<T> From<T> for BigUDecimal
where
  T: super::traits::ToDigits,
{
  fn from(value: T) -> Self {
    Self {
      exponent: 0,
      mantissa: value.into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::compiler::traits::Pow as _;

  use super::*;
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  fn dec(s: &str) -> BigUDecimal {
    s.parse().unwrap()
  }

  fn int(n: u64) -> BigUInt {
    n.into()
  }
  // Helper para comparar con tolerancia
  fn approx_eq(a: &BigUDecimal, b: &BigUDecimal, tol: f64) -> bool {
    let diff = a.sub(b);
    diff.lt(&BigUDecimal::from(tol))
  }

  #[test]
  fn test_lt_one() {
    let a = BigUDecimal::from(0.5);
    let b = BigUDecimal::from(1.0);
    let c = BigUDecimal::from(2.1);

    assert!(a.lt_one());
    assert!(!b.lt_one());
    assert!(!c.lt_one());
  }

  #[test]
  fn test_sqrt_basic() {
    let four = BigUDecimal::from(4.0);
    let nine = BigUDecimal::from(9.0);
    let zero = BigUDecimal::from(0.0);

    let sqrt4 = four.sqrt();
    let sqrt9 = nine.sqrt();
    let sqrt0 = zero.sqrt();

    assert!(approx_eq(&sqrt4, &BigUDecimal::from(2.0), 1e-20));
    assert!(approx_eq(&sqrt9, &BigUDecimal::from(3.0), 1e-20));
    assert!(approx_eq(&sqrt0, &BigUDecimal::from(0.0), 1e-20));
  }

  #[test]
  fn test_exp_known_values() {
    let one = BigUDecimal::from(1.0);
    let two = BigUDecimal::from(2.0);
    let exp1 = one.exp(); // e
    let exp2 = two.exp(); // e²

    let e = BigUDecimal::euler();
    let e2 = e.mul(&e);
    assert!(approx_eq(&exp1, &e, 1e-15));
    assert!(approx_eq(&exp2, &e2, 1e-15));
  }

  #[test]
  fn test_pow_integer_exponent() {
    let two = BigUDecimal::from(2.0);
    let exp3 = BigUInt::from(3u8);
    let result = (&two).pow(&exp3);
    assert!(approx_eq(&result, &BigUDecimal::from(8.0), 1e-15));

    // 5^0 = 1
    let five = BigUDecimal::from(5.0);
    let zero = BigUInt::from(0u8);
    let result = (&five).pow(&zero);
    assert!(approx_eq(&result, &BigUDecimal::from(1.0), 1e-15));
  }

  #[test]
  fn test_exp_growth() {
    let zero = BigUDecimal::from(0.0);
    let one = BigUDecimal::from(1.0);
    let five = BigUDecimal::from(5.0);
    let exp0 = zero.exp();
    let exp1 = one.exp();
    let exp5 = five.exp();

    assert!(approx_eq(&exp0, &BigUDecimal::from(1.0), 1e-20));
    assert!(exp1.lt(&exp5)); // e < e⁵
  }
  #[test]
  fn test_constants() {
    const PI_STR: &str    = "3.141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067982148086513282306647093844609550582231725359408128481117450284102701938521105559644622948954930381964428810975665933446128475648233786783165271201909145648";
    const TAU_STR: &str   = "6.283185307179586476925286766559005768394338798750211641949889184615632812572417997256069650684234135964296173026564613294187689219101164463450718816256962234900568205403877042211119289245897909860763928857621951331866892256951296467573566330542403818291297";
    const EULER_STR: &str = "2.718281828459045235360287471352662497757247093699959574966967627724076630353547594571382178525166427427466391932003059921817413596629043572900334295260595630738132328627943490763233829880753195251019011573834187930702154089149934884167509244761460668082264";
    const LN10: &str      = "2.302585092994045684017991454684364207601101488628772976033327900967572609677352480235997205089598298341967784042286248633409525465082806756666287369098781689482907208325554680843799894826233198528393505308965377732628846163366222287698219886746543667474404";
    assert_eq!(PI_STR.parse::<BigUDecimal>().unwrap(), BigUDecimal::pi());
    assert_eq!(TAU_STR.parse::<BigUDecimal>().unwrap(), BigUDecimal::tau());
    assert_eq!(
      EULER_STR.parse::<BigUDecimal>().unwrap(),
      BigUDecimal::euler()
    );
    assert_eq!(LN10.parse::<BigUDecimal>().unwrap(), BigUDecimal::ln10())
  }

  fn hash_value<T: Hash>(v: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
  }

  #[test]
  fn test_hash_basic() {
    let a = dec("1.0");
    let b = dec("10.00");
    let c = dec("1.00");

    // Normalizamos manualmente para que coincida con hash
    let mut a_norm = a.clone();
    a_norm.normalize();
    let mut b_norm = b.clone();
    b_norm.normalize();
    let mut c_norm = c.clone();
    c_norm.normalize();

    let ha = hash_value(&a);
    let hb = hash_value(&b);
    let hc = hash_value(&c);

    // Hash de números equivalentes después de normalize deberían ser iguales
    assert_eq!(ha, hc, "Hashes de 1.0 y 1.00 deben ser iguales");
    assert_ne!(ha, hb, "Hashes de 1.0 y 10.00 no deben ser iguales");
  }

  #[test]
  fn test_hash_in_hashmap() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let a = dec("1.0");
    let b = dec("1.00");

    map.insert(a.clone(), "uno");
    // b debe reemplazar a
    map.insert(b.clone(), "uno-normalizado");

    assert_eq!(
      map.len(),
      1,
      "Solo debe haber una clave tras insertar equivalente"
    );
    assert_eq!(map.get(&a).unwrap(), &"uno-normalizado");
    assert_eq!(map.get(&b).unwrap(), &"uno-normalizado");
  }

  #[test]
  fn test_normalize() {
    let mut data = BigUDecimal {
      mantissa: int(100100),
      exponent: 3,
    };
    data.normalize();
    assert_eq!(data.to_string(), "1001E-1")
  }
  #[test]
  fn test_cmp_half() {
    let a = dec("1.23"); // 0.23 < 0.5
    assert_eq!(a.cmp_decimals_half(), std::cmp::Ordering::Less);

    let b = dec("2.50"); // 0.50 == 0.5
    assert_eq!(b.cmp_decimals_half(), std::cmp::Ordering::Equal);

    let c = dec("3.75"); // 0.75 > 0.5
    assert_eq!(c.cmp_decimals_half(), std::cmp::Ordering::Greater);

    let d = dec("4.0"); // 0.0 < 0.5
    assert_eq!(d.cmp_decimals_half(), std::cmp::Ordering::Less);

    let e = dec("5."); // sin parte decimal -> Less
    assert_eq!(e.cmp_decimals_half(), std::cmp::Ordering::Less);

    // Caso con muchos decimales
    let f = dec(&format!("1.{}", "0".repeat(254) + "5")); // 0...05 < 0.5
    assert_eq!(f.cmp_decimals_half(), std::cmp::Ordering::Less);
  }

  #[test]
  fn test_addition() {
    let a = dec("12.34");
    let b = dec("0.66");
    let res = &a + &b;
    assert_eq!(res.mantissa.to_string(), "13");
    assert_eq!(res.exponent, 0); // 13.00
  }

  #[test]
  fn test_subtraction() {
    let a = dec("12.34");
    let b = dec("2.34");
    let res = &a - &b;
    assert_eq!(res.mantissa.to_string(), "10");
    assert_eq!(res.exponent, 0); // 10.00
  }

  #[test]
  fn test_multiplication_no_overflow() {
    let a = dec("1.23"); // mantissa 123, exponent 2
    let b = dec("4.56"); // mantissa 456, exponent 2
    let res = &a * &b;
    // 123*456=56088, exponent = 4
    assert_eq!(res.mantissa.to_string(), "56088");
    assert_eq!(res.exponent, 4);
  }

  #[test]
  fn test_multiplication_with_overflow() {
    // No esta normalizado, 100....E-250 realmente es 1E-0
    let large_exp = dec(&format!("1.{}", "0".repeat(250)));
    let res = &large_exp * &large_exp; // 250+250=500 -> rest=245
    assert_eq!(res.mantissa.to_string(), "1"); // mantissa truncada
    assert_eq!(res.exponent, 0); // exponent capped
  }

  #[test]
  fn test_division_no_overflow() {
    let a = dec("12.34");
    let b = dec("2.0");
    let res = &a / &b;
    // 12.34 / 2 = 6.17
    assert_eq!(res.to_string(), format!("617E-2"));
  }

  #[test]
  fn test_division_with_overflow() {
    let large_exp = dec(&format!("1.{}", "0".repeat(250)));
    let res = &large_exp / &large_exp;
    assert_eq!(res.mantissa.to_string(), "1");
    assert_eq!(res.exponent, 0);
  }

  #[test]
  fn test_partial_eq() {
    let a = dec("10.0");
    let b = int(10);
    assert_eq!(a, b);

    let c = dec("10.1");
    assert_ne!(c, b);
  }

  #[test]
  fn test_has_decimals() {
    let a = dec("10.00");
    assert!(!a.has_decimals());
    let b = dec("10.01");
    assert!(b.has_decimals());
  }

  #[test]
  fn test_trunc() {
    let a = dec("123.45");
    let trunc = a.trunc();
    assert_eq!(trunc.to_string(), "123");
  }
}
