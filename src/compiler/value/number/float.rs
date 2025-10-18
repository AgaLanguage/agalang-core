use super::BigUInt;

mod exponent {
  use super::BigUInt;
  use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
  };

  #[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
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
      if dec_len == 0{
        return Ordering::Less;
      }
      let dec_value: BigUInt = dec_part_str.parse().unwrap();
      let half = &format!("5{}", "0".repeat(self.exponent as usize - 1)).parse().unwrap();
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

      Ok(BigUDecimal {
        mantissa: value,
        exponent,
      })
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

      let mut data = BigUDecimal {
        mantissa,
        exponent: lhs.exponent,
      };
      data.normalize();
      data
    }
  }
  impl Sub for &BigUDecimal {
    type Output = BigUDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
      let (lhs, rhs) = establish!(self, rhs);

      let mantissa = &lhs.mantissa - &rhs.mantissa;

      let mut data = BigUDecimal {
        mantissa,
        exponent: lhs.exponent,
      };
      data.normalize();
      data
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

      let mut data = BigUDecimal { mantissa, exponent };
      data.normalize();
      data
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

      let mut data = BigUDecimal { mantissa, exponent };
      data.normalize();
      data
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
  #[cfg(test)]
  mod tests {
    use super::*;

    fn dec(s: &str) -> BigUDecimal {
      s.parse().unwrap()
    }

    fn int(n: u64) -> BigUInt {
      n.to_string().parse().unwrap()
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
}

mod decimal {
  use super::BigUInt;
  use std::{
    cmp::Ordering,
    fmt::Display,
    hash,
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
  };
  const DIVISION_DECIMALS: usize = 100;

  #[derive(Clone, Eq, Debug, Default)]
  pub struct Decimals(Vec<u8>);
  impl Decimals {
    pub fn is_zero(&self) -> bool {
      self.0.iter().all(|&x| x == 0)
    }
    /// Compara con respecto a .5 (utilizado para el redondeo)
    pub fn cmp_half(&self) -> Ordering {
      if self.0.is_empty() {
        return Ordering::Less;
      }

      let first_byte = self.0[0];
      if first_byte < 0x50 {
        Ordering::Less
      } else if first_byte > 0x50 {
        Ordering::Greater
      } else if self.0[1..].iter().all(|&b| b == 0) {
        Ordering::Equal
      } else {
        Ordering::Greater
      }
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
    }
  }
  impl Ord for Decimals {
    fn cmp(&self, other: &Self) -> Ordering {
      for i in 0..self.0.len() {
        if self.0[i] != other.0[i] {
          return self.0[i].cmp(&other.0[i]);
        }
      }
      Ordering::Equal
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
  impl hash::Hash for Decimals {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
      self.to_string().hash(state);
    }
  }

  #[derive(Clone, Eq, Debug, Default)]
  pub struct BigUDecimal {
    integer: BigUInt,
    decimals: Decimals,
  }
  impl BigUDecimal {
    pub fn is_zero(&self) -> bool {
      self.integer.is_zero() && self.decimals.is_zero()
    }
    /// Compara si tiene decimales.
    ///
    /// Cualquier .0 se considera sin decimales
    ///
    /// Este método internamente llama a [`Decimals::cmp_half`].
    pub fn has_decimals(&self) -> bool {
      !self.decimals.is_zero()
    }
    /// Elimina los decimales manteniendo solo la parte entera
    pub fn trunc(&self) -> BigUInt {
      self.integer.clone()
    }
    /// Te da el menor byte de la funcion
    /// Este método internamente llama a [`Decimals::cmp_half`].
    pub fn cmp_decimals_half(&self) -> Ordering {
      self.decimals.cmp_half()
    }
  }
  impl Display for BigUDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}.{}", self.integer, self.decimals)
    }
  }
  impl PartialEq for BigUDecimal {
    fn eq(&self, other: &Self) -> bool {
      self.to_string() == other.to_string()
    }
  }
  impl hash::Hash for BigUDecimal {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
      self.to_string().hash(state);
    }
  }

  impl Add for &BigUDecimal {
    type Output = BigUDecimal;
    fn add(self, rhs: Self) -> Self::Output {
      let (integer, decimals) = add_float(
        (&self.integer, &self.decimals),
        (&rhs.integer, &rhs.decimals),
      );
      BigUDecimal { integer, decimals }
    }
  }
  impl Add<&BigUInt> for &BigUDecimal {
    type Output = BigUDecimal;
    fn add(self, rhs: &BigUInt) -> Self::Output {
      let (integer, decimals) =
        add_float((&self.integer, &self.decimals), (rhs, &Decimals::default()));
      BigUDecimal { integer, decimals }
    }
  }

  impl Sub for &BigUDecimal {
    type Output = BigUDecimal;
    fn sub(self, rhs: Self) -> Self::Output {
      let (integer, decimals) = sub_float(
        (&self.integer, &self.decimals),
        (&rhs.integer, &rhs.decimals),
      );
      BigUDecimal { integer, decimals }
    }
  }
  impl Sub<&BigUInt> for &BigUDecimal {
    type Output = BigUDecimal;
    fn sub(self, rhs: &BigUInt) -> Self::Output {
      let (integer, decimals) =
        sub_float((&self.integer, &self.decimals), (rhs, &Decimals::default()));
      BigUDecimal { integer, decimals }
    }
  }

  impl Mul for &BigUDecimal {
    type Output = BigUDecimal;
    fn mul(self, rhs: Self) -> Self::Output {
      let (x, x_dec, y, y_dec) = (&self.integer, &self.decimals, &rhs.integer, &rhs.decimals);
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

      BigUDecimal {
        integer: int_part.into(),
        decimals: frac_part.into(),
      }
    }
  }
  impl Mul<&BigUInt> for &BigUDecimal {
    type Output = BigUDecimal;
    fn mul(self, rhs: &BigUInt) -> Self::Output {
      let (x, x_dec, y) = (&self.integer, &self.decimals, rhs);
      let x_dec = x_dec.to_string();
      let x_str = format!("{}{}", x, x_dec);
      let x: BigUInt = x_str.into();
      let mut decimals = x_dec.len();

      let result = &x * y;
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

      BigUDecimal {
        integer: int_part.into(),
        decimals: frac_part.into(),
      }
    }
  }

  impl Div for &BigUDecimal {
    type Output = BigUDecimal;
    fn div(self, rhs: Self) -> Self::Output {
      let (x, x_dec, y, y_dec) = (&self.integer, &self.decimals, &rhs.integer, &rhs.decimals);
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

      BigUDecimal {
        integer: int_part.into(),
        decimals: frac_part.into(),
      }
    }
  }
  impl Div<&BigUInt> for &BigUDecimal {
    type Output = BigUDecimal;
    fn div(self, rhs: &BigUInt) -> Self::Output {
      let (x, x_dec, y) = (&self.integer, &self.decimals, rhs);
      let mut x_dec = x_dec.to_string();
      let mut y_dec = String::new();
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

      BigUDecimal {
        integer: int_part.into(),
        decimals: frac_part.into(),
      }
    }
  }

  impl Rem for &BigUDecimal {
    type Output = BigUDecimal;
    fn rem(self, rhs: Self) -> Self::Output {
      let binding = self / rhs;
      let div = binding.trunc();
      let mul = rhs * &div;
      self - &mul
    }
  }
  impl PartialOrd for BigUDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
    }
  }
  impl Ord for BigUDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
      let (x, x_dec, y, y_dec) = (
        &self.integer,
        &self.decimals,
        &other.integer,
        &other.decimals,
      );
      let cmp = x.cmp(y);
      if cmp == Ordering::Equal {
        x_dec.cmp(y_dec)
      } else {
        cmp
      }
    }
  }

  impl PartialEq<BigUInt> for BigUDecimal {
    fn eq(&self, other: &BigUInt) -> bool {
      if self.integer != *other {
        return false;
      }
      self.decimals.is_zero()
    }
  }
  impl PartialOrd<BigUInt> for BigUDecimal {
    fn partial_cmp(&self, other: &BigUInt) -> Option<Ordering> {
      let (x, x_dec, y) = (&self.integer, &self.decimals, other);
      let cmp = x.cmp(y);
      Some(if cmp == Ordering::Equal && !x_dec.is_zero() {
        Ordering::Greater
      } else {
        cmp
      })
    }
  }
  impl From<String> for BigUDecimal {
    fn from(value: String) -> Self {
      let split = value.split('.');
      let int_part = split.clone().nth(0).unwrap_or("0").to_string();
      let frac_part = split.clone().nth(1).unwrap_or("0").to_string();
      Self {
        integer: int_part.into(),
        decimals: frac_part.into(),
      }
    }
  }
  impl FromStr for BigUDecimal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      Ok(s.to_string().into())
    }
  }

  fn add_float(
    (x, x_dec): (&BigUInt, &Decimals),
    (y, y_dec): (&BigUInt, &Decimals),
  ) -> (BigUInt, Decimals) {
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
    (&(x + y) + &BigUInt::from(carry), z_dec)
  }
  fn sub_float(
    (x, x_dec): (&BigUInt, &Decimals),
    (y, y_dec): (&BigUInt, &Decimals),
  ) -> (BigUInt, Decimals) {
    let mut x_dec = x_dec.to_string();
    let mut y_dec = y_dec.to_string();
    let decimals = x_dec.len().max(y_dec.len());
    x_dec.extend(std::iter::repeat_n('0', decimals - x_dec.len()));
    y_dec.extend(std::iter::repeat_n('0', decimals - y_dec.len()));

    let x_full = BigUInt::from(format!("{}{}", x, x_dec));
    let y_full = BigUInt::from(format!("{}{}", y, y_dec));
    let z = match x_full.cmp(&y_full) {
      Ordering::Greater => &x_full - &y_full,
      Ordering::Less => &y_full - &x_full,
      Ordering::Equal => return (BigUInt::from(0u8), Decimals(vec![])),
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

    (BigUInt::from(z), Decimals::from(z_dec))
  }
}
pub use exponent::BigUDecimal;
