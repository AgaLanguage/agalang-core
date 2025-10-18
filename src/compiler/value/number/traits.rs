/// Convierte un numero estandar en un vector de bytes little-endian
/// Utilizado para Big256
pub trait ToDigits {
  fn to_digits(self) -> Vec<u8>;
}

/// convierte un vector de bytes en T
/// donde T es un tipo entero sin signo
trait FromDigits {
  fn from_digits(vec: Vec<u8>) -> Self;
}

macro_rules! impl_to_digits {
  ($($t:ty),*) => {
    $(
      impl ToDigits for $t {
        fn to_digits(self) -> Vec<u8> {
          self.to_le_bytes().to_vec()
        }
      }
      impl FromDigits for $t {
        fn from_digits(vec: Vec<u8>) -> Self {
          let mut arr = [0u8; std::mem::size_of::<$t>()];

          let len = vec.len().min(arr.len());
          arr[..len].copy_from_slice(&vec[..len]);

          <$t>::from_le_bytes(arr)
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
    fn test_from_digits_u8() {
        let bytes = vec![42];
        let x = u8::from_digits(bytes);
        assert_eq!(x, 42);
    }

    #[test]
    fn test_to_digits_u32() {
        let x: u32 = 0x12345678;
        let bytes = x.to_digits();
        assert_eq!(bytes, vec![0x78, 0x56, 0x34, 0x12]); // little-endian
    }

    #[test]
    fn test_from_digits_u32() {
        let bytes = vec![0x78, 0x56, 0x34, 0x12];
        let x = u32::from_digits(bytes);
        assert_eq!(x, 0x12345678);
    }

    #[test]
    fn test_small_to_large() {
        // u8 -> u32
        let x: u8 = 7;
        let y = u32::from_digits(x.to_digits());
        assert_eq!(y, 7);
    }

    #[test]
    fn test_large_to_small_truncate() {
        // u32 -> u8
        let x: u32 = 0x12345678;
        let y = u8::from_digits(x.to_digits());
        assert_eq!(y, 0x78); // toma solo el primer byte (little-endian)
    }

    #[test]
    fn test_partial_bytes_fill() {
        // menos bytes que el tipo, rellenar con ceros
        let bytes = vec![0xAA];
        let x = u32::from_digits(bytes);
        assert_eq!(x, 0xAA);
    }

    #[test]
    fn test_partial_bytes_truncate() {
        // más bytes que el tipo, truncar
        let bytes = vec![1, 2, 3, 4, 5, 6];
        let x: u32 = FromDigits::from_digits(bytes);
        assert_eq!(x, 0x04030201); // solo primeros 4 bytes
    }
}