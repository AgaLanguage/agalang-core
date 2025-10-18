/// Convierte un numero estandar en un vector de bytes little-endian
/// Utilizado para Big256
pub trait ToDigits {
  fn to_digits(self) -> Vec<u8>;
}

macro_rules! impl_to_digits {
  ($($t:ty),*) => {
    $(
      impl ToDigits for $t {
        fn to_digits(self) -> Vec<u8> {
          self.to_le_bytes().to_vec()
        }
      }
    )*
  };
}

// Implementamos para los tipos deseados
impl_to_digits!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_digits_u8() {
        let x: u8 = 42;
        let bytes = x.to_digits();
        assert_eq!(bytes, vec![42]);
    }

    #[test]
    fn test_to_digits_u32() {
        let x: u32 = 0x12345678;
        let bytes = x.to_digits();
        assert_eq!(bytes, vec![0x78, 0x56, 0x34, 0x12]); // little-endian
    }
}