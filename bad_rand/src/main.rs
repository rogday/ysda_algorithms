use rayon::prelude::*;

fn main() {
    const TARGET: &[u8; 10] = b"NLXGI4NoAp";
    const ALPHABET: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const FORWARD_BY: usize = 100_000_000 + 310;
    const STATE_SIZE: usize = 31;
    const MODULO: i64 = i32::MAX as i64;

    let mut pows: [i64; STATE_SIZE] = [1; STATE_SIZE];
    (1..STATE_SIZE).for_each(|i| pows[i] = (pows[i - 1] * 16_807) % MODULO);
    let pows = pows;

    let mut forward_matrix = [[0u32; STATE_SIZE]; STATE_SIZE];
    (0..STATE_SIZE).for_each(|i| forward_matrix[i][i] = 1);

    let mut begin = 3;
    let mut end = 0;

    for _ in 0..FORWARD_BY {
        (0..STATE_SIZE).for_each(|j| {
            forward_matrix[begin][j] = forward_matrix[begin][j].wrapping_add(forward_matrix[end][j])
        });
        begin = (begin + 1) % STATE_SIZE;
        end = (end + 1) % STATE_SIZE;
    }

    (1u32..=std::u32::MAX).into_par_iter().for_each(|seed| {
        if seed % 100_000_000 == 0 {
            println!("{:?}: {}M", std::thread::current().id(), seed / 1_000_000);
        }

        for i in 0..TARGET.len() {
            let coeffs = &forward_matrix[(begin + i) - TARGET.len()];
            let first_part = coeffs[0].wrapping_mul(((seed as i32 as i64) * pows[0]) as u32);

            let sum: u32 = coeffs
                .iter()
                .zip(pows.iter())
                .skip(1)
                .map(|(x, y)| x.wrapping_mul((((seed as i64) * y) % MODULO) as u32))
                .fold(first_part, |acc, x| acc.wrapping_add(x));

            if TARGET[i] != ALPHABET[(sum >> 1) as usize % ALPHABET.len()] {
                return;
            }
        }

        println!("\n{:?} found: {}\n", std::thread::current().id(), seed);
    });
}
