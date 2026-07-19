//! BogoPoW proof generation and verification.
//! See specifications/consensus.md 5.3-5.6.

use crate::header::BlockHeader;
use bogosortcoin_primitives::Hash256;
use sha3::digest::XofReader;
use sha3::{Sha3_256, Shake256};

const SEED_DOMAIN: &[u8] = b"BOGOPOW-v1/seed";
const PERMUTATION_DOMAIN: &[u8] = b"BOGOPOW-v1/permutation";
const TICKET_DOMAIN: &[u8] = b"BOGOPOW-v1/ticket";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub seed: Hash256,
    pub permutation: Vec<u16>,
    pub ticket: Hash256,
}

pub fn derive_seed(header: &BlockHeader) -> Hash256 {
    use sha3::Digest;
    let mut hasher = Sha3_256::new();
    hasher.update(SEED_DOMAIN);
    hasher.update(header.canonical_bytes());
    hasher.finalize().into()
}

pub fn derive_ticket(seed: &Hash256) -> Hash256 {
    use sha3::Digest;
    let mut hasher = Sha3_256::new();
    hasher.update(TICKET_DOMAIN);
    hasher.update(seed);
    hasher.finalize().into()
}

/// Draws a uniform integer in `[0, range)` from a XOF stream via rejection
/// sampling, so no modulo bias is introduced (specifications/consensus.md 5.4).
fn uniform_below(reader: &mut impl XofReader, range: u64) -> u64 {
    assert!(range > 0);
    let zone = u64::MAX - (u64::MAX % range);
    loop {
        let mut buf = [0u8; 8];
        reader.read(&mut buf);
        let value = u64::from_be_bytes(buf);
        if value < zone {
            return value % range;
        }
    }
}

/// Generates the deterministic permutation for `seed` via Fisher-Yates
/// applied from the final position to the first.
pub fn generate_permutation(seed: &Hash256, n: u16) -> Vec<u16> {
    use sha3::digest::{ExtendableOutput, Update};

    let mut permutation: Vec<u16> = (0..n).collect();
    let mut shake = Shake256::default();
    shake.update(PERMUTATION_DOMAIN);
    shake.update(seed);
    let mut reader = shake.finalize_xof();

    for i in (1..n as usize).rev() {
        let j = uniform_below(&mut reader, (i + 1) as u64) as usize;
        permutation.swap(i, j);
    }
    permutation
}

pub fn is_sorted(permutation: &[u16]) -> bool {
    permutation.iter().enumerate().all(|(i, v)| *v as usize == i)
}

/// Returns true iff `ticket`, interpreted as a big-endian unsigned integer,
/// does not exceed `target`.
pub fn ticket_meets_target(ticket: &Hash256, target: &Hash256) -> bool {
    ticket.as_slice() <= target.as_slice()
}

pub fn generate_proof(header: &BlockHeader) -> Proof {
    let seed = derive_seed(header);
    let permutation = generate_permutation(&seed, header.permutation_size);
    let ticket = derive_ticket(&seed);
    Proof { seed, permutation, ticket }
}

pub fn is_valid(header: &BlockHeader) -> bool {
    let proof = generate_proof(header);
    is_sorted(&proof.permutation) && ticket_meets_target(&proof.ticket, &header.difficulty_target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with_nonce(n: u16, nonce: u64, target: Hash256) -> BlockHeader {
        BlockHeader {
            network_id: 1,
            protocol_version: 1,
            height: 0,
            previous_block_hash: [0u8; 32],
            transaction_merkle_root: [0u8; 32],
            state_root: [0u8; 32],
            timestamp: 0,
            difficulty_target: target,
            permutation_size: n,
            miner_commitment: [0u8; 32],
            extra_nonce: 0,
            nonce,
        }
    }

    #[test]
    fn seed_derivation_is_deterministic() {
        let header = header_with_nonce(8, 1, [0xff; 32]);
        assert_eq!(derive_seed(&header), derive_seed(&header));
    }

    #[test]
    fn different_nonces_give_different_seeds() {
        let a = header_with_nonce(8, 1, [0xff; 32]);
        let b = header_with_nonce(8, 2, [0xff; 32]);
        assert_ne!(derive_seed(&a), derive_seed(&b));
    }

    #[test]
    fn permutation_is_a_valid_permutation_of_n() {
        let header = header_with_nonce(8, 12345, [0xff; 32]);
        let seed = derive_seed(&header);
        let mut perm = generate_permutation(&seed, 8);
        perm.sort();
        assert_eq!(perm, (0u16..8).collect::<Vec<_>>());
    }

    #[test]
    fn permutation_generation_is_deterministic() {
        let header = header_with_nonce(10, 777, [0xff; 32]);
        let seed = derive_seed(&header);
        assert_eq!(generate_permutation(&seed, 10), generate_permutation(&seed, 10));
    }

    #[test]
    fn easiest_target_always_meets_ticket() {
        let ticket = [0xaa; 32];
        assert!(ticket_meets_target(&ticket, &[0xff; 32]));
    }

    #[test]
    fn zero_target_rejects_any_nonzero_ticket() {
        let ticket = [0x01; 32];
        assert!(!ticket_meets_target(&ticket, &[0x00; 32]));
    }

    #[test]
    fn mining_a_small_permutation_eventually_succeeds() {
        // N = 4 => success probability 1/4! per attempt; an easy target makes
        // the ticket condition near-certain so this exercises the permutation
        // search converging within a bounded number of nonces.
        let target = [0xff; 32];
        let mut nonce = 0u64;
        loop {
            let header = header_with_nonce(4, nonce, target);
            if is_valid(&header) {
                break;
            }
            nonce += 1;
            assert!(nonce < 1_000_000, "did not find a valid proof in time");
        }
    }
}
