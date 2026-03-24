//! Utility functions for HarmonyPIR.
//!
//! Primarily XOR operations on variable-length byte entries.

/// XOR `src` into `dst` in place: `dst[i] ^= src[i]` for all i.
///
/// # Panics
/// Panics if `dst` and `src` have different lengths.
pub fn xor_bytes_into(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len(), "XOR operands must have equal length");
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d ^= *s;
    }
}

/// Return the XOR of two byte slices as a new Vec.
///
/// # Panics
/// Panics if the slices have different lengths.
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len(), "XOR operands must have equal length");
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// A zero entry of `w` bytes, used as the identity for XOR.
pub fn zero_entry(w: usize) -> Vec<u8> {
    vec![0u8; w]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_bytes_into() {
        let mut a = vec![0xFF, 0x00, 0xAA];
        let b = vec![0x0F, 0xF0, 0x55];
        xor_bytes_into(&mut a, &b);
        assert_eq!(a, vec![0xF0, 0xF0, 0xFF]);
    }

    #[test]
    fn test_xor_self_is_zero() {
        let a = vec![1, 2, 3, 4];
        let result = xor_bytes(&a, &a);
        assert_eq!(result, vec![0, 0, 0, 0]);
    }
}
