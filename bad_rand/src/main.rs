use rayon::prelude::*;
use std::cell::RefCell;

fn main() {
    const TARGET: &[u8; 10] = b"NLXGI4NoAp";
    const ALPHABET: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const FORWARD_BY: usize = 100_000_000 + 310;
    const STATE_SIZE: usize = 31;

    let mut forward_matrix = [[0u32; STATE_SIZE]; STATE_SIZE];
    (0..STATE_SIZE).for_each(|i| forward_matrix[i][i] = 1);

    let mut begin = 3;
    let mut end = 0;

    for _ in 0..FORWARD_BY {
        (0..STATE_SIZE).for_each(|j| forward_matrix[begin][j] += forward_matrix[end][j]);
        begin = (begin + 1) % STATE_SIZE;
        end = (end + 1) % STATE_SIZE;
    }

    (1u32..=std::u32::MAX).into_par_iter().for_each(|seed| {
        if seed % 100_000_000 == 0 {
            println!("{:?}: {}M", std::thread::current().id(), seed / 1_000_000);
        }

        thread_local! {
            static INNER_STATE: RefCell<[i32; STATE_SIZE]> = RefCell::new([0i32; STATE_SIZE]);
        }

        INNER_STATE.with(|state| {
            let mut word = seed as i64;
            for x in state.borrow_mut().iter_mut() {
                *x = word as i32;
                word = (16_807 * word) % 2_147_483_647;
            }

            for i in 0..TARGET.len() {
                let sum: u32 = forward_matrix[begin - TARGET.len() + i]
                    .iter()
                    .zip(state.borrow().iter())
                    .map(|(x, &y)| x * (y as u32))
                    .sum();

                if TARGET[i] != ALPHABET[(sum >> 1) as usize % ALPHABET.len()] {
                    return;
                }
            }

            println!("\n{:?} found: {}\n", std::thread::current().id(), seed);
        });
    });
}
