//! Alternate endings as Guitar Pro keeps them: per measure, a bitmask of
//! the passes it's played on. Shared by the legacy model and the optimized
//! model's reading of scores saved with them.

/// The 1-based passes in a GP alternate-ending bitmask (bit 0 = pass 1).
pub fn ending_mask_numbers(mask: u8) -> Vec<u8> {
    (0u8..8)
        .filter(|&bit| mask & (1 << bit) != 0)
        .map(|bit| bit + 1)
        .collect()
}

/// Inverse of [`ending_mask_numbers`]: passes beyond 1–8 are left out.
pub fn ending_numbers_mask(numbers: &[u8]) -> u8 {
    numbers
        .iter()
        .filter(|&&n| (1..=8).contains(&n))
        .fold(0, |mask, n| mask | 1 << (n - 1))
}
