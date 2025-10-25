use std::{
  cmp::Ordering,
  fmt, hash,
  ops::{Add, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign},
};

/// Representa un número en base 256.
/// Cada elemento del Vec es un "dígito" en base 256.
/// Formato little-endian (mas facil de manejar)
#[derive(Clone, Debug)]
pub struct Big256 {
  digits: Vec<u8>,
}

impl Big256 {
  pub fn new(digits: Vec<u8>) -> Self {
    let mut d = Self { digits };
    d.normalize();
    d
  }
  pub fn unit(&self) -> &u8 {
    self.digits.first().unwrap_or(&0)
  }
  pub fn is_zero(&self) -> bool {
    self.digits.iter().all(|&x| x == 0)
  }
  pub fn normalize(&mut self) {
    while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
      self.digits.pop();
    }
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
  /// Retorna 10^n como BigUInt
  pub fn pow10(n: u8) -> Self {
    let mut result = Self::from(1u8);
    let mut base = Self::from(10u8);
    let mut exp = n;

    while exp > 0 {
      if exp % 2 == 1 {
        result *= &base;
      }

      base = &base * &base;
      exp /= 2;
    }

    result
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
    Self::new(vec![0])
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
    self.digits.hash(state);
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
    let a32 = bytes_to_u32_vec(&self.digits);
    let b32 = bytes_to_u32_vec(&other.digits);
    let mut res = Vec::new();
    let mut borrow = 0u64;
    for i in 0..a32.len().max(b32.len()) {
      let a = *a32.get(i).unwrap_or(&0) as i64;
      let b = *b32.get(i).unwrap_or(&0) as i64;
      let mut diff = a - b - (borrow as i64);
      if diff < 0 {
        diff += (1u64 << 32) as i64; // pedimos prestado del siguiente bloque
        borrow = 1;
      } else {
        borrow = 0;
      }
      res.push(diff as u32);
    }

    // Reconstruye el vector de bytes desde los u32
    let mut res_bytes = Vec::new();
    for block in res {
      res_bytes.extend_from_slice(&block.to_le_bytes());
    }

    Big256::new(res_bytes)
  }
}
impl Add for &Big256 {
  type Output = Big256;
  fn add(self, other: Self) -> Self::Output {
    // Convierte los bytes en bloques de u32
    let a32 = bytes_to_u32_vec(&self.digits);
    let b32 = bytes_to_u32_vec(&other.digits);
    let mut res_blocks = Vec::new();
    let mut carry = 0u64;
    for i in 0..a32.len().max(b32.len()) {
      let a = *a32.get(i).unwrap_or(&0) as u64;
      let b = *b32.get(i).unwrap_or(&0) as u64;
      let sum = a + b + carry;
      res_blocks.push((sum & 0xFFFF_FFFF) as u32); // 32 bits
      carry = sum >> 32;
    }
    if carry > 0 {
      res_blocks.push(carry as u32);
    }

    // Reconstruye el vector de bytes desde los u32
    let mut res_bytes = Vec::new();
    for block in res_blocks {
      res_bytes.extend_from_slice(&block.to_le_bytes());
    }

    Big256::new(res_bytes)
  }
}
impl Mul for &Big256 {
  type Output = Big256;

  fn mul(self, rhs: Self) -> Self::Output {
    let a32 = bytes_to_u32_vec(&self.digits);
    let b32 = bytes_to_u32_vec(&rhs.digits);

    let mut res = vec![0u32; a32.len() + b32.len()];

    for (i, &a) in a32.iter().enumerate() {
      let mut carry = 0u32;
      for (j, &b) in b32.iter().enumerate() {
        let idx = i + j;
        let prod = res[idx] as u64 + a as u64 * b as u64 + carry as u64;
        res[idx] = (prod & 0xFFFF_FFFF) as u32;
        carry = (prod >> 32) as u32;
      }
      if carry > 0 {
        res[i + b32.len()] += carry;
      }
    }

    // convertir a bytes y recortar ceros sobrantes
    let mut out = Vec::new();
    for n in res {
      out.extend_from_slice(&n.to_le_bytes());
    }

    // recortamos al tamaño mínimo requerido para no perder dígitos
    let digits_len = self.digits.len() + rhs.digits.len();
    Big256::new(out.into_iter().take(digits_len).collect())
  }
}
impl Div for &Big256 {
  type Output = Big256;

  fn div(self, rhs: Self) -> Self::Output {
    // Esta realmente resta, y eso ya esta optimzado en u32
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
      return Big256::default();
    }

    let mut quotient = Big256 {
      digits: vec![0; dividend.digits.len()],
    };
    let mut remainder = Big256::default();

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
        remainder -= &divisor;

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
impl Rem for &Big256 {
  type Output = Big256;

  fn rem(self, other: Self) -> Self::Output {
    assert!(!other.is_zero(), "division by zero");

    // Si self < other, el residuo es self
    if self < other {
      return self.clone();
    }

    // Clonamos valores
    let mut dividend = self.clone();
    let divisor = other.clone();

    // Repetimos resta hasta que dividend < divisor
    while dividend >= divisor {
      dividend -= &divisor;
    }

    dividend
  }
}

impl SubAssign<&Big256> for Big256 {
  fn sub_assign(&mut self, rhs: &Self) {
    *self = &*self - rhs
  }
}
impl DivAssign<&Big256> for Big256 {
  fn div_assign(&mut self, rhs: &Big256) {
    *self = &*self / rhs
  }
}
impl MulAssign<&Big256> for Big256 {
  fn mul_assign(&mut self, rhs: &Big256) {
    *self = &*self * rhs
  }
}

impl std::str::FromStr for Big256 {
  type Err = super::NumberError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use super::traits::FromStrRadix;
    Self::from_str_radix(s, 10)
  }
}
impl super::traits::FromStrRadix for Big256 {
  fn from_str_radix(src: &str, radix: u8) -> Result<Self, super::NumberError> {
    if !(2..=36).contains(&radix) {
      return Err(super::NumberError::Radix(radix));
    }
    let s = src.trim();
    if s.is_empty() {
      return Err(super::NumberError::Empty);
    }

    let mut digits = vec![0u8];

    for c in s.chars() {
      if !c.is_ascii_digit() {
        return Err(super::NumberError::InvalidCharacter(c));
      }

      // Malabares binarios
      let val = match c {
        '0'..='9' => c as u8 - b'0',
        'a'..='z' => c as u8 - b'a' + 10,
        'A'..='Z' => c as u8 - b'A' + 10,
        _ => return Err(super::NumberError::InvalidCharacter(c)),
      };

      if val >= radix {
        return Err(super::NumberError::InvalidDigit(c, radix));
      }

      // Multiplicamos el numero actual por la base
      let mut carry = val as u16;
      for d in digits.iter_mut() {
        let tmp = (*d as u16) * (radix as u16) + carry;
        *d = (tmp & 0xFF) as u8;
        carry = tmp >> 8;
      }
      if carry > 0 {
        digits.push(carry as u8);
      }
    }
    Ok(Big256::new(digits))
  }
}

impl<T> From<T> for Big256
where
  T: super::traits::ToDigits,
{
  fn from(value: T) -> Self {
    Self::new(value.to_digits())
  }
}

fn bytes_to_u32_vec(data: &[u8]) -> Vec<u32> {
  let slice_len = data.len() - (data.len() % 4);
  // Solo por seguridad no quiero romper algo
  let slice = if slice_len == 0 {
    &[]
  } else {
    unsafe {
      let slice = &data[0..slice_len];
      // Como se queja clippy con sus "buenas practicas"
      &*core::ptr::slice_from_raw_parts(slice as *const [u8] as *const [u8; 4], slice_len / 4)
    }
  };
  let remainder = &data[slice_len..];

  let mut vec_of_chunks: Vec<[u8; 4]> = slice.to_vec();

  // Copia los bytes sobrantes al último chunk
  if !remainder.is_empty() {
    let mut last = [0; 4];
    last[..remainder.len()].copy_from_slice(remainder);
    vec_of_chunks.push(last);
  }

  // Convertimos los chunks en u32
  vec_of_chunks.into_iter().map(u32::from_le_bytes).collect()
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
    let digits = 50;
    let a = &Big256 {
      digits: vec![1; digits],
    };
    let b = &from_u64(4);
    let r = a * b;
    assert_eq!(
      r,
      Big256 {
        digits: vec![4; digits]
      }
    );
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
    let n: Big256 = "12345".parse().unwrap();
    assert_eq!(n.to_string(), "12345");
  }

  #[test]
  fn test_fromstr_leading_zeros() {
    let n: Big256 = "000123".parse().unwrap();
    assert_eq!(n.to_string(), "123");
  }
}
