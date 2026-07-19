use bogosortcoin_primitives::{CanonicalDecode, CanonicalEncode, DecodeError, Hash256};

/// Block header fields per specifications/consensus.md 5.1.
///
/// All integers are encoded big-endian; all hashes are raw 32-byte values.
/// The header has no variable-length fields, so canonical encoding is a
/// fixed-order concatenation with no length prefixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub network_id: u32,
    pub protocol_version: u32,
    pub height: u64,
    pub previous_block_hash: Hash256,
    pub transaction_merkle_root: Hash256,
    pub state_root: Hash256,
    pub timestamp: u64,
    pub difficulty_target: Hash256,
    pub permutation_size: u16,
    pub miner_commitment: Hash256,
    pub extra_nonce: u64,
    pub nonce: u64,
}

impl CanonicalEncode for BlockHeader {
    fn encode(&self, out: &mut Vec<u8>) {
        self.network_id.encode(out);
        self.protocol_version.encode(out);
        self.height.encode(out);
        self.previous_block_hash.encode(out);
        self.transaction_merkle_root.encode(out);
        self.state_root.encode(out);
        self.timestamp.encode(out);
        self.difficulty_target.encode(out);
        self.permutation_size.encode(out);
        self.miner_commitment.encode(out);
        self.extra_nonce.encode(out);
        self.nonce.encode(out);
    }
}

impl CanonicalDecode for BlockHeader {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(BlockHeader {
            network_id: u32::decode(input)?,
            protocol_version: u32::decode(input)?,
            height: u64::decode(input)?,
            previous_block_hash: Hash256::decode(input)?,
            transaction_merkle_root: Hash256::decode(input)?,
            state_root: Hash256::decode(input)?,
            timestamp: u64::decode(input)?,
            difficulty_target: Hash256::decode(input)?,
            permutation_size: u16::decode(input)?,
            miner_commitment: Hash256::decode(input)?,
            extra_nonce: u64::decode(input)?,
            nonce: u64::decode(input)?,
        })
    }
}

impl BlockHeader {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.encode(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BlockHeader {
        BlockHeader {
            network_id: 1,
            protocol_version: 1,
            height: 42,
            previous_block_hash: [1u8; 32],
            transaction_merkle_root: [2u8; 32],
            state_root: [3u8; 32],
            timestamp: 1_753_000_000,
            difficulty_target: [0xff; 32],
            permutation_size: 8,
            miner_commitment: [4u8; 32],
            extra_nonce: 7,
            nonce: 99,
        }
    }

    #[test]
    fn decode_of_encode_is_identity() {
        let header = sample();
        let bytes = header.canonical_bytes();
        let mut cursor = bytes.as_slice();
        let decoded = BlockHeader::decode(&mut cursor).unwrap();
        assert!(cursor.is_empty());
        assert_eq!(decoded, header);
    }

    #[test]
    fn encode_of_decode_is_identity() {
        let header = sample();
        let bytes = header.canonical_bytes();
        let mut cursor = bytes.as_slice();
        let decoded = BlockHeader::decode(&mut cursor).unwrap();
        assert_eq!(decoded.canonical_bytes(), bytes);
    }

    #[test]
    fn truncated_header_is_rejected() {
        let header = sample();
        let mut bytes = header.canonical_bytes();
        bytes.truncate(bytes.len() - 1);
        let mut cursor = bytes.as_slice();
        assert_eq!(BlockHeader::decode(&mut cursor), Err(DecodeError::UnexpectedEnd));
    }
}
