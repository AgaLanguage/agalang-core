pub trait ToDigits {
  fn to_digits(self) -> Vec<u8>;
}
impl ToDigits for u8 {
  fn to_digits(self) -> Vec<u8> {
    vec![self]
  }
}
impl ToDigits for u16 {
  fn to_digits(self) -> Vec<u8> {
    self.to_le_bytes().to_vec()
  }
}
impl ToDigits for u32 {
  fn to_digits(self) -> Vec<u8> {
    self.to_le_bytes().to_vec()
  }
}
impl ToDigits for u64 {
  fn to_digits(self) -> Vec<u8> {
    self.to_le_bytes().to_vec()
  }
}
impl ToDigits for u128 {
  fn to_digits(self) -> Vec<u8> {
    self.to_le_bytes().to_vec()
  }
}
impl ToDigits for usize {
  fn to_digits(self) -> Vec<u8> {
    self.to_le_bytes().to_vec()
  }
}