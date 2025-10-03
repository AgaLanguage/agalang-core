use std::{hash,cmp::Ordering, fmt, ops::{Add, Div, Mul, Rem, Sub, SubAssign}};

/// Representa un número en base 256.
/// Cada elemento del Vec es un "dígito" en base 256.
/// Formato little-indian (mas facil de manejar)
#[derive(Clone, Debug)]
pub struct Big256{digits:Vec<u8>}

impl Big256 {
  pub fn new() -> Self {
    Self{digits:vec![]}
  }
  pub fn is_zero(&self) -> bool {
    if self.digits.is_empty() {
      return true;
    }
    self.digits.iter().all(|&x| x == 0)
  }
  fn normalize(&mut self){
    while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
      self.digits.pop();
    }
  }
  pub fn last(&self) -> &u8 {
    self.digits.first().unwrap_or(&0)
  }
}
impl From<u8> for Big256 {
  fn from(value: u8) -> Self {
    let mut data = Self{digits:vec![value]};
    data.normalize();
    data
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
    Self::new()
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

impl Ord for Big256{
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
impl Add for &Big256{
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
#[cfg(test)]
mod tests {
    use super::*;

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
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Eq, Hash, Debug)]
pub struct BCDUInt(Vec<u8>);
impl BCDUInt {
  pub fn last(&self) -> &u8 {
    self.0.last().unwrap_or(&0)
  }
  pub fn is_zero(&self) -> bool {
    if self.0.is_empty() {
      return true;
    }
    self.0.iter().all(|&x| x == 0)
  }
  pub fn trim_leading_zeros(mut self) -> Self {
    while self.0.len() > 1 && self.0[0] == 0 {
      self.0.remove(0);
    }
    self
  }
  pub fn digits(&self) -> Vec<u8> {
    self
      .to_string()
      .chars()
      .map(|c| c.to_digit(10).unwrap() as u8)
      .collect()
  }
  pub fn from_digits(digits: Vec<u8>) -> Self {
    let mut vec = Vec::new();
    let mut i = digits.len();
    while i > 0 {
      i -= 1;
      let low = digits[i];
      let high = if i == 0 {
        0
      } else {
        i -= 1;
        digits[i]
      };
      vec.push((high << 4) | low);
    }
    vec.reverse();
    Self(vec)
  }
  fn compare_digits(a: &[u8], b: &[u8]) -> i8 {
    if a.len() > b.len() {
      1
    } else if a.len() < b.len() {
      -1
    } else {
      for (&x, &y) in a.iter().zip(b) {
        if x > y {
          return 1;
        };
        if x < y {
          return -1;
        };
      }
      0
    }
  }
  fn sub_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut carry = 0;
    let mut ai = a.len() as isize - 1;
    let mut bi = b.len() as isize - 1;

    while ai >= 0 {
      let mut ad = a[ai as usize] as i8 - carry;
      let bd = if bi >= 0 { b[bi as usize] as i8 } else { 0 };

      if ad < bd {
        ad += 10;
        carry = 1;
      } else {
        carry = 0;
      }

      result.push((ad - bd) as u8);
      ai -= 1;
      bi -= 1;
    }

    result.reverse();
    while result.first() == Some(&0) && result.len() > 1 {
      result.remove(0);
    }

    result
  }
}

impl fmt::Display for BCDUInt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut result = String::new();
    for i in 0..self.0.len() {
      let byte = self.0[i];
      let (byte_1, byte_0) = ((byte & 0xF0) >> 4, byte & 0x0F);
      result.push_str(&format!("{:X}{:X}", byte_1, byte_0));
    }
    let trim_result = result.trim_start_matches('0');
    write!(
      f,
      "{}",
      if trim_result.is_empty() {
        "0"
      } else {
        trim_result
      }
    )
  }
}
impl PartialEq for BCDUInt {
  fn eq(&self, other: &Self) -> bool {
    self.to_string() == other.to_string()
  }
}

impl SubAssign<&BCDUInt> for BCDUInt {
  fn sub_assign(&mut self, rhs: &Self) {
    *self = &*self - rhs;
  }
}

impl Add for &BCDUInt {
  type Output = BCDUInt;
  fn add(self, rhs: Self) -> Self::Output {
    let lhs = &self.0;
    let rhs = &rhs.0;

    let mut result = Vec::new();
    let mut carry = 0;

    let max_len = lhs.len().max(rhs.len());

    for i in 0..max_len {
      let a = *lhs.get(lhs.len().wrapping_sub(1 + i)).unwrap_or(&0);
      let b = *rhs.get(rhs.len().wrapping_sub(1 + i)).unwrap_or(&0);

      // Extraer nibbles altos y bajos
      let (a1, a0) = ((a >> 4) & 0x0F, a & 0x0F);
      let (b1, b0) = ((b >> 4) & 0x0F, b & 0x0F);

      let mut sub0 = a0 + b0 + carry;
      carry = 0;
      if sub0 >= 10 {
        sub0 -= 10;
        carry = 1;
      }

      let mut sub1 = a1 + b1 + carry;
      carry = 0;
      if sub1 >= 10 {
        sub1 -= 10;
        carry = 1;
      }

      // restar 10 es suficiente y menos costoso

      result.push(((sub1) << 4) | sub0);
    }

    // Eliminar ceros a la izquierda, excepto si es cero solo
    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }

    result.reverse();
    BCDUInt(result)
  }
}
impl Sub for &BCDUInt {
  type Output = BCDUInt;

  fn sub(self, rhs: Self) -> Self::Output {
    let lhs = &self.0;
    let rhs = &rhs.0;

    let mut result = Vec::new();
    let mut carry = 0;

    // Asumimos que lhs >= rhs comprobacion
    let max_len = lhs.len().max(rhs.len());

    for i in 0..max_len {
      let a = *lhs.get(lhs.len().wrapping_sub(1 + i)).unwrap_or(&0);
      let b = *rhs.get(rhs.len().wrapping_sub(1 + i)).unwrap_or(&0);

      // Extraer nibbles altos y bajos
      let (a1, a0) = ((a >> 4) & 0x0F, a & 0x0F);
      let (b1, b0) = ((b >> 4) & 0x0F, b & 0x0F);

      // Resta del nibble bajo
      let mut sub0 = a0 as i8 - b0 as i8 - carry;
      carry = 0;
      if sub0 < 0 {
        sub0 += 10;
        carry = 1;
      }

      // Resta del nibble alto
      let mut sub1 = a1 as i8 - b1 as i8 - carry;
      carry = 0;
      if sub1 < 0 {
        sub1 += 10;
        carry = 1;
      }

      result.push(((sub1 as u8) << 4) | (sub0 as u8));
    }

    // Eliminar ceros a la izquierda, excepto si es cero solo
    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }

    result.reverse();
    BCDUInt(result)
  }
}
impl Mul for &BCDUInt {
  type Output = BCDUInt;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut x_digits = self.digits();
    let mut y_digits = rhs.digits();
    x_digits.reverse();
    y_digits.reverse();

    let mut result = vec![0u8; x_digits.len() + y_digits.len()];

    for (i, &x) in x_digits.iter().enumerate() {
      for (j, &y) in y_digits.iter().enumerate() {
        result[i + j] += x * y;
        if result[i + j] >= 10 {
          result[i + j + 1] += result[i + j] / 10;
          result[i + j] %= 10;
        }
      }
    }

    while result.len() > 1 && *result.last().unwrap() == 0 {
      result.pop();
    }
    result.reverse();

    BCDUInt::from_digits(result)
  }
}
impl Div for &BCDUInt {
  type Output = BCDUInt;

  fn div(self, rhs: Self) -> Self::Output {
    let x_digits = self.digits();
    let y_digits = rhs.digits();
    if y_digits.is_empty() || y_digits.iter().all(|&d| d == 0) {
      panic!("División por cero");
    }

    let mut result = Vec::new();
    let mut remainder = Vec::new();

    for digit in x_digits {
      remainder.push(digit);
      while remainder.first() == Some(&0) && remainder.len() > 1 {
        remainder.remove(0);
      }

      let mut count = 0;
      while BCDUInt::compare_digits(&remainder, &y_digits) >= 0 {
        remainder = BCDUInt::sub_digits(&remainder, &y_digits);
        count += 1;
      }

      result.push(count);
    }

    while result.first() == Some(&0) && result.len() > 1 {
      result.remove(0);
    }

    BCDUInt::from_digits(result)
  }
}
impl Rem for &BCDUInt {
  type Output = BCDUInt;
  fn rem(self, rhs: Self) -> Self::Output {
    let div = &(self / rhs);
    let mul = &(rhs * div);
    self - mul
  }
}
impl PartialOrd for BCDUInt {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for BCDUInt {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let x_digits = self.digits();
    let y_digits = other.digits();
    let x_len = x_digits.len();
    let y_len = y_digits.len();
    if x_len > y_len {
      return std::cmp::Ordering::Greater;
    } else if x_len < y_len {
      return std::cmp::Ordering::Less;
    }
    let max = x_len.max(y_len);
    for i in 0..max {
      let x = if x_len > i { x_digits[i] } else { 0 };
      let y = if y_len > i { y_digits[i] } else { 0 };
      let value = x.cmp(&y);
      if value != std::cmp::Ordering::Equal {
        return value;
      }
    }
    std::cmp::Ordering::Equal
  }
}

impl From<String> for BCDUInt {
  fn from(value: String) -> Self {
    let digits = value.chars().map(|c| c as u8 - b'0').collect();
    Self::from_digits(digits)
  }
}
impl From<u8> for BCDUInt {
  fn from(value: u8) -> Self {
    if value > 99 {
      panic!("Value must be between 0 and 99");
    }
    let high = value / 10;
    let low = value % 10;
    let byte = (high << 4) | low;
    BCDUInt(vec![byte])
  }
}
