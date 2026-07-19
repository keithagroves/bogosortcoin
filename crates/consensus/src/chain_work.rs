//! Chain work calculation. See specifications/consensus.md 5.7.

use bogosortcoin_primitives::Hash256;
use num_bigint::BigUint;
use num_traits::One;

fn factorial(n: u16) -> BigUint {
    let mut result = BigUint::one();
    for i in 2..=n as u64 {
        result *= i;
    }
    result
}

/// `floor(N! * 2^256 / (difficulty_target + 1))`, using unbounded integer
/// arithmetic to avoid overflow regardless of `N` or the target value.
pub fn block_work(permutation_size: u16, difficulty_target: &Hash256) -> BigUint {
    let target = BigUint::from_bytes_be(difficulty_target);
    let denominator = target + BigUint::one();
    let numerator = factorial(permutation_size) * (BigUint::one() << 256);
    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_target_reduces_to_factorial_alone() {
        let work = block_work(8, &[0xff; 32]);
        assert_eq!(work, factorial(8));
    }

    #[test]
    fn zero_target_is_hardest_and_largest() {
        let easy = block_work(8, &[0xff; 32]);
        let hard = block_work(8, &[0x00; 32]);
        assert!(hard > easy);
    }

    #[test]
    fn work_decreases_monotonically_as_target_grows() {
        let smaller_target = block_work(8, &{
            let mut t = [0u8; 32];
            t[31] = 0x10;
            t
        });
        let larger_target = block_work(8, &{
            let mut t = [0u8; 32];
            t[31] = 0x20;
            t
        });
        assert!(smaller_target > larger_target);
    }

    #[test]
    fn larger_permutation_size_increases_work() {
        let n8 = block_work(8, &[0xff; 32]);
        let n9 = block_work(9, &[0xff; 32]);
        assert!(n9 > n8);
    }
}
