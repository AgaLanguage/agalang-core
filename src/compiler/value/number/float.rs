use super::BigUInt;

use std::{
  cmp::Ordering,
  fmt::Display,
  hash,
  ops::{Add, Div, Mul, Sub},
  str::FromStr,
};

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
  pub fn new(mantissa: BigUInt, exponent: u8) -> Self {
    let mut d = Self { mantissa, exponent };
    d.normalize();
    d
  }
  pub fn is_zero(&self) -> bool {
    self.mantissa.is_zero()
  }
  /// Compara la parte decimal de `self` con X.5
  pub fn cmp_decimals_half(&self) -> Ordering {
    if !self.has_decimals() {
      return Ordering::Less; // no hay parte decimal
    }

    let mantissa_string = self.mantissa.to_string();

    let dec_part_str = if mantissa_string.len() >= self.exponent as usize {
      &mantissa_string[(mantissa_string.len() - self.exponent as usize)..]
    } else {
      &mantissa_string
    };
    let dec_len = dec_part_str.len();
    if dec_len == 0 {
      return Ordering::Less;
    }
    let dec_value: BigUInt = dec_part_str.parse().unwrap();
    let half = &format!("5{}", "0".repeat(self.exponent as usize - 1))
      .parse()
      .unwrap();
    dec_value.cmp(half)
  }
  /// Compara si tiene decimales.
  pub fn has_decimals(&self) -> bool {
    let dec = self.exponent as usize;
    if dec == 0 {
      return false;
    }
    let s = self.mantissa.to_string();
    s[(s.len() - dec)..].chars().any(|c| c != '0')
  }
  /// Elimina los decimales manteniendo solo la parte entera
  pub fn trunc(&self) -> BigUInt {
    let s = self.mantissa.to_string();
    let len = s.len();
    let dec = self.exponent as usize;
    if dec >= len { "0" } else { &s[..(len - dec)] }
      .to_string()
      .into()
  }
  fn normalize(&mut self) {
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
  fn establish(&self, exponent: u8) -> Self {
    let mantissa = self.to_absolute(exponent - self.exponent);
    Self { mantissa, exponent }
  }
  fn to_absolute(&self, exponent: u8) -> BigUInt {
    let mut s = self.mantissa.to_string();
    s.extend(std::iter::repeat_n('0', exponent as usize));
    s.parse().unwrap()
  }
}

impl FromStr for BigUDecimal {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(".").collect();
    if parts.len() != 2 {
      return Err(format!("Invalid format '{s}'"));
    }
    let int_part = parts[0];
    let mut dec_part = parts[1];
    if dec_part.len() > 255 {
      dec_part = &dec_part[..255]
    }

    let value = format!("{int_part}{dec_part}").parse()?;
    let exponent = dec_part.len() as u8;

    Ok(BigUDecimal::new(value, exponent))
  }
}
impl From<String> for BigUDecimal {
  fn from(value: String) -> Self {
    value.parse().unwrap()
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
      (255u8, (pre_exponent - 255) as usize)
    } else {
      (pre_exponent as u8, 0)
    };

    // Truncamos mantissa si hay exceso de decimales
    if rest > 0 {
      let divisor_str = format!("1{}", "0".repeat(rest));
      let divisor: BigUInt = divisor_str.parse().unwrap();
      mantissa = &mantissa / &divisor;
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
      (255u8, (pre_exponent - 255) as usize)
    } else {
      (pre_exponent as u8, 0)
    };

    // Truncamos mantissa si hay exceso de decimales
    if rest > 0 {
      let divisor_str = format!("1{}", "0".repeat(rest));
      let divisor: BigUInt = divisor_str.parse().unwrap();
      mantissa = &mantissa / &divisor;
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

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  fn dec(s: &str) -> BigUDecimal {
    s.parse().unwrap()
  }

  fn int(n: u64) -> BigUInt {
    n.to_string().parse().unwrap()
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
