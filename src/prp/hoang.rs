//! Optimized small-domain PRP based on Hoang, Morris, and Rogaway (Crypto 2012).
//!
//! # Overview
//!
//! This implements Algorithm 5 from the HarmonyPIR paper: an optimized variant of the
//! Hoang et al. card-shuffle that reduces AES calls by 4× through **phase grouping**.
//!
//! ## How the card shuffle works (Algorithm 4, original)
//!
//! A random permutation over [N'] is viewed as shuffling N' cards. Over `r` rounds:
//! 1. A round key K[i] ∈ [N'] is sampled.
//! 2. Cards at positions X and X ⊕ K[i] are paired.
//! 3. They swap if the round function F_i(min(X, X')) = 1.
//!
//! To track a single card at position X, we only need to follow X through each round
//! (we don't track other cards). This takes O(r) AES calls.
//!
//! ## Phase optimization (Algorithm 5, our implementation)
//!
//! Rounds are grouped into phases of β = 4 rounds. Within a phase:
//!
//! 1. Compute the **phase group** G: the 2^β = 16 positions reachable from X
//!    by XORing subsets of the β round keys. Sort G.
//! 2. A single AES call on G[0] produces 128 bits. Each of the 16 positions
//!    in G is assigned β = 4 bits, giving 64 bits total (fits in 128-bit AES output).
//! 3. For each round within the phase, the round function for any pair is read
//!    from the appropriate bit of the AES output.
//!
//! This reduces AES calls from `r` to `r/β = r/4`.
//!
//! ## Round key derivation
//!
//! Round keys and round functions are derived from a single AES key `k`:
//! - Round key: `K[i] = AES_k("key" || i) mod N'`
//! - Round function: via phase group (see above)
//!
//! ## Security
//!
//! From Lemma 6.1: HarmonyPIR only needs an (N'/2, ε)-secure PRP, because the
//! server sees at most half the permutation evaluations. With r = Θ(log N' + |log ε|)
//! rounds, this is achieved.

use aes::cipher::{BlockEncrypt, KeyInit};
use aes::{Aes128, Block};

use super::Prp;

/// Small-domain PRP using the optimized Hoang et al. card shuffle (Algorithm 5).
///
/// Suitable for HarmonyPIR0. Security reduces to AES.
pub struct HoangPrp {
    /// Domain size N'. Valid inputs are 0..N'-1.
    domain_size: usize,
    /// Number of rounds. Must be a multiple of BETA.
    r: usize,
    /// Precomputed round keys: K[i] ∈ [0, domain_size).
    round_keys: Vec<usize>,
    /// The AES cipher instance, used for deriving round functions.
    cipher: Aes128,
}

/// Phase size: number of rounds per phase.
const BETA: usize = 4;

impl HoangPrp {
    /// Create a new small-domain PRP.
    ///
    /// - `domain_size`: the permutation domain [N'].
    /// - `r`: number of rounds (must be a multiple of BETA = 4).
    /// - `key`: 16-byte AES key.
    pub fn new(domain_size: usize, r: usize, key: &[u8; 16]) -> Self {
        assert!(r % BETA == 0, "r must be a multiple of BETA={BETA}");
        assert!(domain_size >= 2, "domain must be >= 2");

        let cipher = Aes128::new(key.into());

        // Precompute round keys: K[i] = AES_k("key" || i) mod domain_size
        let round_keys: Vec<usize> = (0..r)
            .map(|i| {
                let plaintext = round_key_plaintext(i);
                let mut block = Block::from(plaintext);
                cipher.encrypt_block(&mut block);
                // Interpret first 8 bytes as u64, mod domain_size.
                let val = u64::from_le_bytes(block[..8].try_into().unwrap());
                (val % domain_size as u64) as usize
            })
            .collect();

        HoangPrp {
            domain_size,
            r,
            round_keys,
            cipher,
        }
    }

    /// Execute the shuffle on position X for a range of phases, in given direction.
    /// `forward = true` means phases 0..num_phases, forward rounds.
    /// `forward = false` means phases in reverse, rounds within each phase in reverse.
    fn shuffle(&self, mut x: usize, forward: bool) -> usize {
        let num_phases = self.r / BETA;

        let phase_iter: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new(0..num_phases)
        } else {
            Box::new((0..num_phases).rev())
        };

        for phase_idx in phase_iter {
            let round_base = phase_idx * BETA;

            // Collect the β round keys for this phase.
            let phase_keys: [usize; BETA] = [
                self.round_keys[round_base],
                self.round_keys[round_base + 1],
                self.round_keys[round_base + 2],
                self.round_keys[round_base + 3],
            ];

            // Compute the phase group G (2^β = 16 positions reachable from X).
            // G[mask] = X ⊕ XOR of keys selected by the bitmask.
            let mut group = [0usize; 1 << BETA];
            for mask in 0..(1u32 << BETA) {
                let mut val = x;
                for bit in 0..BETA {
                    if mask & (1 << bit) != 0 {
                        val ^= phase_keys[bit];
                    }
                }
                group[mask as usize] = val;
            }

            // Sort G to get a canonical ordering.
            let mut sorted_group = group;
            sorted_group.sort_unstable();

            // Single AES call for this phase's round functions.
            // f = AES_k("function" || phase_idx || G_sorted[0])
            let func_plaintext = round_func_plaintext(phase_idx, sorted_group[0]);
            let mut func_block = Block::from(func_plaintext);
            self.cipher.encrypt_block(&mut func_block);
            let f_bits = u128::from_le_bytes(func_block.into());

            // Apply each round within the phase.
            let round_iter: Box<dyn Iterator<Item = usize>> = if forward {
                Box::new(0..BETA)
            } else {
                Box::new((0..BETA).rev())
            };

            for j in round_iter {
                let x_prime = x ^ phase_keys[j];
                let x_min = x.min(x_prime);

                // rank(x_min, sorted_group): find x_min's position in the sorted group.
                let p = sorted_group
                    .iter()
                    .position(|&v| v == x_min)
                    .expect("x_min must be in the phase group");

                // The round function bit is at position (p * BETA + j) in f_bits.
                let bit_index = p * BETA + j;
                let should_swap = (f_bits >> bit_index) & 1 == 1;

                if should_swap {
                    x = x_prime;
                }
            }
        }

        x
    }
}

impl Prp for HoangPrp {
    fn forward(&self, x: usize) -> usize {
        assert!(x < self.domain_size, "input {x} >= domain {}", self.domain_size);
        self.shuffle(x, true)
    }

    fn inverse(&self, y: usize) -> usize {
        assert!(y < self.domain_size, "input {y} >= domain {}", self.domain_size);
        // The swap-or-not structure is self-inverse per round:
        // to invert, just reverse the order of rounds (and phases).
        self.shuffle(y, false)
    }

    fn domain(&self) -> usize {
        self.domain_size
    }
}

#[cfg(feature = "alf")]
impl super::BatchPrp for HoangPrp {
    fn batch_forward(&self) -> Vec<usize> {
        use rayon::prelude::*;
        (0..self.domain_size)
            .into_par_iter()
            .map(|x| self.shuffle(x, true))
            .collect()
    }
}

#[cfg(not(feature = "alf"))]
impl super::BatchPrp for HoangPrp {
    fn batch_forward(&self) -> Vec<usize> {
        (0..self.domain_size)
            .map(|x| self.shuffle(x, true))
            .collect()
    }
}

/// Build the 16-byte AES plaintext for round key derivation.
/// Format: "key\0" (4 bytes) || round_index (4 bytes LE) || padding (8 bytes of 0).
fn round_key_plaintext(round: usize) -> [u8; 16] {
    let mut buf = [0u8; 16];
    buf[0..3].copy_from_slice(b"key");
    buf[4..8].copy_from_slice(&(round as u32).to_le_bytes());
    buf
}

/// Build the 16-byte AES plaintext for the round function.
/// Format: "fn\0" (3 bytes) || phase_index (4 bytes LE) || g0 (4 bytes LE) || padding.
fn round_func_plaintext(phase: usize, g0: usize) -> [u8; 16] {
    let mut buf = [0u8; 16];
    buf[0..2].copy_from_slice(b"fn");
    buf[3..7].copy_from_slice(&(phase as u32).to_le_bytes());
    buf[7..11].copy_from_slice(&(g0 as u32).to_le_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_inverse_roundtrip() {
        let key = [0x42u8; 16];
        let domain = 64;
        let r = 44; // must be multiple of 4
        let prp = HoangPrp::new(domain, r, &key);

        for x in 0..domain {
            let y = prp.forward(x);
            assert!(y < domain, "forward({x}) = {y} out of range");
            let x_back = prp.inverse(y);
            assert_eq!(x_back, x, "inverse(forward({x})) = {x_back} != {x}");
        }
    }

    #[test]
    fn test_is_permutation() {
        let key = [0xAB; 16];
        let domain = 32;
        let r = 44;
        let prp = HoangPrp::new(domain, r, &key);

        let mut outputs: Vec<usize> = (0..domain).map(|x| prp.forward(x)).collect();
        outputs.sort();
        let expected: Vec<usize> = (0..domain).collect();
        assert_eq!(outputs, expected, "forward is not a permutation");
    }

    #[test]
    fn test_different_keys_different_permutations() {
        let domain = 16;
        let r = 44;
        let prp1 = HoangPrp::new(domain, r, &[1u8; 16]);
        let prp2 = HoangPrp::new(domain, r, &[2u8; 16]);

        let out1: Vec<usize> = (0..domain).map(|x| prp1.forward(x)).collect();
        let out2: Vec<usize> = (0..domain).map(|x| prp2.forward(x)).collect();
        assert_ne!(out1, out2, "different keys should give different permutations");
    }
}
