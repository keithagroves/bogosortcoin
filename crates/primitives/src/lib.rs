//! Fixed-width primitive types and canonical big-endian encoding used by
//! consensus-critical structures. See specifications/consensus.md 5.2.

pub type Hash256 = [u8; 32];

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEnd,
}

pub trait CanonicalEncode {
    fn encode(&self, out: &mut Vec<u8>);
}

pub trait CanonicalDecode: Sized {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError>;
}

macro_rules! impl_canonical_uint {
    ($t:ty) => {
        impl CanonicalEncode for $t {
            fn encode(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_be_bytes());
            }
        }

        impl CanonicalDecode for $t {
            fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
                const SIZE: usize = std::mem::size_of::<$t>();
                if input.len() < SIZE {
                    return Err(DecodeError::UnexpectedEnd);
                }
                let (bytes, rest) = input.split_at(SIZE);
                *input = rest;
                Ok(<$t>::from_be_bytes(bytes.try_into().unwrap()))
            }
        }
    };
}

impl_canonical_uint!(u16);
impl_canonical_uint!(u32);
impl_canonical_uint!(u64);

impl CanonicalEncode for Hash256 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}

impl CanonicalDecode for Hash256 {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        if input.len() < 32 {
            return Err(DecodeError::UnexpectedEnd);
        }
        let (bytes, rest) = input.split_at(32);
        *input = rest;
        Ok(bytes.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_uints() {
        let mut buf = Vec::new();
        42u32.encode(&mut buf);
        1234567890123u64.encode(&mut buf);
        let mut cursor = buf.as_slice();
        assert_eq!(u32::decode(&mut cursor).unwrap(), 42u32);
        assert_eq!(u64::decode(&mut cursor).unwrap(), 1234567890123u64);
        assert!(cursor.is_empty());
    }

    #[test]
    fn roundtrip_hash() {
        let h: Hash256 = [7u8; 32];
        let mut buf = Vec::new();
        h.encode(&mut buf);
        let mut cursor = buf.as_slice();
        assert_eq!(Hash256::decode(&mut cursor).unwrap(), h);
    }

    #[test]
    fn decode_rejects_truncated_input() {
        let buf = [0u8; 3];
        let mut cursor = buf.as_slice();
        assert_eq!(u32::decode(&mut cursor), Err(DecodeError::UnexpectedEnd));
    }
}
