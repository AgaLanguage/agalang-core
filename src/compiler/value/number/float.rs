use super::BigUInt;

mod decimal {
  use super::BigUInt;
  use std::{
    cmp::Ordering,
    fmt::Display,
    hash,
    ops::{Add, Div, Mul, Rem, Sub},
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
    pub fn trunc(&self) -> &BigUInt {
      &self.integer
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
      let mul = rhs * div;
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
pub use decimal::BigUDecimal;
