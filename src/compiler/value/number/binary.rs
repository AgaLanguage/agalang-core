use std::{
  cmp::Ordering,
  fmt, hash,
  ops::{Add, Div, Mul, Sub},
  str::FromStr,
};

/// Representa un número en base 256.
/// Cada elemento del Vec es un "dígito" en base 256.
/// Formato little-indian (mas facil de manejar)
#[derive(Clone, Debug)]
pub struct Big256 {
  digits: Vec<u8>,
}

impl Big256 {
  pub fn unit(&self) -> &u8 {
    self.digits.first().unwrap_or(&0)
  }
  pub fn is_zero(&self) -> bool {
    self.digits.iter().all(|&x| x == 0)
  }
  fn normalize(&mut self) {
    while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
      self.digits.pop();
    }
  }
  pub fn last(&self) -> &u8 {
    self.digits.first().unwrap_or(&0)
  }
  fn shl1(&mut self) {
    let mut carry = 0;
    for byte in self.digits.iter_mut() {
      let new_carry = (*byte & 0x80) >> 7;
      *byte = (*byte << 1) | carry;
      carry = new_carry;
    }
    if carry != 0 {
      self.digits.push(carry);
    }
  }
}
impl fmt::Display for Big256 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Caso especial: cero
    if self.digits.iter().all(|&d| d == 0) {
      return write!(f, "0");
    }

    // Copiamos los dígitos porque haremos divisiones destructivas
    let mut digits = self.digits.clone();

    // Vector de caracteres decimales (en orden inverso)
    let mut decimal_digits = Vec::new();

    // División repetida por 10
    while digits.iter().any(|&d| d != 0) {
      let mut carry = 0u16;
      for d in digits.iter_mut().rev() {
        let cur = (*d as u16) + (carry << 8);
        *d = (cur / 10) as u8;
        carry = cur % 10;
      }
      decimal_digits.push((b'0' + carry as u8) as char);
    }

    // Invertimos y unimos
    decimal_digits.reverse();
    let result: String = decimal_digits.into_iter().collect();

    write!(f, "{}", result)
  }
}

impl Default for Big256 {
  fn default() -> Self {
    Self{ digits: vec![0] }
  }
}
impl PartialEq for Big256 {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for Big256 {}
impl hash::Hash for Big256 {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    let mut norm = self.clone();
    norm.normalize();
    norm.digits.hash(state);
  }
}

impl Ord for Big256 {
  fn cmp(&self, other: &Self) -> Ordering {
    let mut a = self.clone();
    let mut b = other.clone();
    a.normalize();
    b.normalize();

    // primero por longitud
    if a.digits.len() < b.digits.len() {
      return Ordering::Less;
    } else if a.digits.len() > b.digits.len() {
      return Ordering::Greater;
    }

    // si tienen misma longitud, comparar desde el dígito más significativo
    for i in (0..a.digits.len()).rev() {
      if a.digits[i] < b.digits[i] {
        return Ordering::Less;
      } else if a.digits[i] > b.digits[i] {
        return Ordering::Greater;
      }
    }

    Ordering::Equal
  }
}
impl PartialOrd for Big256 {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Sub for &Big256 {
  type Output = Big256;

  fn sub(self, other: Self) -> Self::Output {
    let mut res = Vec::new();
    let mut borrow = 0i16;
    for i in 0..self.digits.len() {
      let a = self.digits[i] as i16;
      let b = *other.digits.get(i).unwrap_or(&0) as i16;
      let mut diff = a - b - borrow;
      if diff < 0 {
        diff += 256;
        borrow = 1;
      } else {
        borrow = 0;
      }
      res.push(diff as u8);
    }
    let mut r = Self::Output { digits: res };
    r.normalize();
    r
  }
}
impl Add for &Big256 {
  type Output = Big256;
  fn add(self, other: Self) -> Self::Output {
    let mut res = Vec::new();
    let mut carry = 0u16;
    let n = self.digits.len().max(other.digits.len());
    for i in 0..n {
      let a = *self.digits.get(i).unwrap_or(&0) as u16;
      let b = *other.digits.get(i).unwrap_or(&0) as u16;
      let sum = a + b + carry;
      res.push((sum & 0xFF) as u8);
      carry = sum >> 8;
    }
    if carry > 0 {
      res.push(carry as u8);
    }
    Self::Output { digits: res }
  }
}
impl Mul for &Big256 {
  type Output = Big256;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut res = vec![0u16; self.digits.len() + rhs.digits.len()];

    // Multiplicación base 256
    for (i, &a) in self.digits.iter().enumerate() {
      let mut carry = 0u16;
      for (j, &b) in rhs.digits.iter().enumerate() {
        let idx = i + j;
        let prod = a as u16 * b as u16 + res[idx] + carry;
        res[idx] = prod & 0xFF;
        carry = prod >> 8;
      }
      if carry > 0 {
        res[i + rhs.digits.len()] += carry;
      }
    }

    let digits = res.into_iter().map(|v| v as u8).collect::<Vec<_>>();
    let mut out = Big256 { digits };
    out.normalize();
    out
  }
}
impl Div for &Big256 {
  type Output = Big256;

  fn div(self, rhs: Self) -> Self::Output {
    // división entre cero no permitida
    if rhs.digits.iter().all(|&d| d == 0) {
      panic!("Division by zero");
    }

    let mut dividend = self.clone();
    let mut divisor = rhs.clone();
    dividend.normalize();
    divisor.normalize();

    // si dividend < divisor → resultado 0
    if dividend < divisor {
      return Big256 { digits: vec![0] };
    }

    let mut quotient = Big256 {
      digits: vec![0; dividend.digits.len()],
    };
    let mut remainder = Big256 { digits: vec![0] };

    let total_bits = dividend.digits.len() * 8;

    // recorremos los bits de más significativo a menos
    for i in (0..total_bits).rev() {
      // shift left del resto 1 bit
      remainder.shl1();

      // añadir el bit i del dividendo al LSB del remainder
      let byte_index = i / 8;
      let bit_index = i % 8;
      let bit = (dividend.digits[byte_index] >> bit_index) & 1;
      remainder.digits[0] |= bit;

      // si remainder >= divisor, restamos divisor y ponemos 1 en cociente
      if remainder >= divisor {
        remainder = &remainder - &divisor;

        // colocar el 1 en la posición correcta del cociente
        let q_byte = i / 8;
        let q_bit = i % 8;
        if q_byte >= quotient.digits.len() {
          quotient.digits.resize(q_byte + 1, 0);
        }
        quotient.digits[q_byte] |= 1 << q_bit;
      }
    }

    quotient.normalize();
    quotient
  }
}

impl FromStr for Big256 {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.trim();
    if s.is_empty() {
      return Err("Cannot parse empty string".to_string());
    }

    let mut digits = vec![0u8];

    for c in s.chars() {
      if !c.is_ascii_digit() {
        return Err(format!("Invalid character '{}'", c));
      }

      // Malabares binarios
      let val = (c as u8 - b'0') as u16;

      // Multiplicamos el numero actual por la base
      let mut carry = val;
      for d in digits.iter_mut() {
        let tmp = (*d as u16) * 10 + carry;
        *d = (tmp & 0xFF) as u8;
        carry = tmp >> 8;
      }
      if carry > 0 {
        digits.push(carry as u8);
      }
    }

    let mut res = Big256 { digits };
    res.normalize();
    Ok(res)
  }
}
impl From<String> for Big256 {
  fn from(value: String) -> Self {
    value.parse().unwrap()
  }
}
impl<T> From<T> for Big256 where T: super::traits::ToDigits {
  fn from(value: T) -> Self {
    Self { digits: value.to_digits() }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  fn from_u64(mut n: u64) -> Big256 {
    let mut digits = Vec::new();
    while n > 0 {
      digits.push((n & 0xFF) as u8);
      n >>= 8;
    }
    if digits.is_empty() {
      digits.push(0);
    }
    Big256 { digits }
  }

  fn to_u64(b: &Big256) -> u64 {
    let mut n = 0u64;
    for (i, &d) in b.digits.iter().enumerate() {
      n |= (d as u64) << (8 * i);
    }
    n
  }

  #[test]
  fn test_unit() {
    assert_eq!(*from_u64(1124).unit(), 100);
    assert_eq!(*from_u64(43208).unit(), 200);
  }

  #[test]
  fn test_add_simple() {
    let a = from_u64(5);
    let b = from_u64(7);
    let res = &a + &b;
    assert_eq!(to_u64(&res), 12);
  }

  #[test]
  fn test_add_with_carry() {
    let a = from_u64(255);
    let b = from_u64(1);
    let res = &a + &b;
    assert_eq!(to_u64(&res), 256);
  }

  #[test]
  fn test_add_multi_byte() {
    let a = from_u64(500);
    let b = from_u64(600);
    let res = &a + &b;
    assert_eq!(to_u64(&res), 1100);
  }

  #[test]
  fn test_sub_simple() {
    let a = from_u64(10);
    let b = from_u64(3);
    let res = &a - &b;
    assert_eq!(to_u64(&res), 7);
  }

  #[test]
  fn test_sub_with_borrow() {
    let a = from_u64(256);
    let b = from_u64(1);
    let res = &a - &b;
    assert_eq!(to_u64(&res), 255);
  }

  #[test]
  fn test_sub_equal() {
    let a = from_u64(12345);
    let b = from_u64(12345);
    let res = &a - &b;
    assert_eq!(to_u64(&res), 0);
  }

  #[test]
  fn test_add_and_sub_inverse() {
    let a = from_u64(99999);
    let b = from_u64(12345);
    let sum = &a + &b;
    let diff = &sum - &b;
    assert_eq!(to_u64(&diff), to_u64(&a));
  }

  #[test]
  fn test_mul_simple() {
    let a = &from_u64(12);
    let b = &from_u64(3);
    let r = a * b;
    assert_eq!(to_u64(&r), 36);
  }

  #[test]
  fn test_mul_large() {
    let a = &from_u64(1234);
    let b = &from_u64(5678);
    let r = a * b;
    assert_eq!(to_u64(&r), 1234 * 5678);
  }

  #[test]
  fn test_div_simple() {
    let a = &from_u64(10);
    let b = &from_u64(2);
    let r = a / b;
    assert_eq!(to_u64(&r), 5);
  }

  #[test]
  fn test_div_large() {
    let a = &from_u64(123456789);
    let b = &from_u64(123);
    let r = a / b;
    assert_eq!(to_u64(&r), 123456789 / 123);
  }

  #[test]
  fn test_div_by_larger() {
    let a = &from_u64(100);
    let b = &from_u64(1000);
    let r = a / b;
    assert_eq!(to_u64(&r), 0);
  }

  #[test]
  #[should_panic]
  fn test_div_zero() {
    let a = &from_u64(42);
    let b = &from_u64(0);
    let _ = a / b;
  }

  #[test]
  fn test_fromstr_simple() {
    let n = Big256::from_str("12345").unwrap();
    assert_eq!(n.to_string(), "12345");
  }

  #[test]
  fn test_fromstr_leading_zeros() {
    let n = Big256::from_str("000123").unwrap();
    assert_eq!(n.to_string(), "123");
  }
}
