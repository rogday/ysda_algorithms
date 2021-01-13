use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
    ops::{Add, Mul, Sub},
};

use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

#[derive(Debug, Copy, Clone)]
struct WrappingNumber<const PRIME: u64>(u64);

impl<const PRIME: u64> Add for WrappingNumber<PRIME> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        WrappingNumber((self.0 + rhs.0) % PRIME)
    }
}

impl<const PRIME: u64> Sub for WrappingNumber<PRIME> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        WrappingNumber((PRIME + self.0 - rhs.0) % PRIME)
    }
}

impl<const PRIME: u64> Mul for WrappingNumber<PRIME> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        WrappingNumber((self.0 * rhs.0) % PRIME)
    }
}

#[derive(Debug, Copy, Clone)]
struct Square {
    x:    usize,
    y:    usize,
    size: usize,
}

impl Square {
    fn equal(
        input: &[Vec<u32>],
        Square { x: lx, y: ly, size }: Square,
        Square { x: rx, y: ry, .. }: Square,
    ) -> bool {
        for i in 0..size {
            if input[ly - i][lx + 1 - size..=lx] != input[ry - i][rx + 1 - size..=rx] {
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
struct Hasher<const PRIME: u64> {
    p_pows: Vec<WrappingNumber<PRIME>>,
    q_pows: Vec<WrappingNumber<PRIME>>,

    matrix: Vec<Vec<WrappingNumber<PRIME>>>,
}

impl<const PRIME: u64> Hasher<PRIME> {
    fn new(input: &[Vec<u32>], p: u64, q: u64) -> Self {
        let p = WrappingNumber(p);
        let q = WrappingNumber(q);

        let n = input.len();
        let m = input[0].len();

        let mut matrix = vec![vec![WrappingNumber(0); m + 1]; n + 1];

        for y in 1..=n {
            for x in 1..=m {
                matrix[y][x] = WrappingNumber(input[y - 1][x - 1] as u64)
                    + matrix[y][x - 1] * p
                    + matrix[y - 1][x] * q
                    - matrix[y - 1][x - 1] * p * q;
            }
        }

        let size = n.min(m) + 1;

        let mut p_pows = vec![WrappingNumber(1); size];
        let mut q_pows = vec![WrappingNumber(1); size];

        for i in 1..size {
            p_pows[i] = p_pows[i - 1] * p;
            q_pows[i] = q_pows[i - 1] * q;
        }

        Self { p_pows, q_pows, matrix }
    }

    fn hash(&self, Square { mut x, mut y, size }: Square) -> WrappingNumber<PRIME> {
        x += 1;
        y += 1;

        let p = self.p_pows[size];
        let q = self.q_pows[size];

        let big = self.matrix[y][x];

        let left = self.matrix[y][x - size];
        let upper = self.matrix[y - size][x];
        let little = self.matrix[y - size][x - size];

        big + little * p * q - left * p - upper * q
    }
}

struct HashCombinator<const PRIME: u64, const N: usize> {
    hashers: [Hasher<PRIME>; N],
}

impl<const PRIME: u64, const N: usize> HashCombinator<PRIME, N> {
    fn new(input: &[Vec<u32>]) -> Self {
        let mut rng = thread_rng();
        let distr = Uniform::new(1, PRIME);

        Self {
            hashers: (0..N)
                .map(|_| Hasher::new(input, distr.sample(&mut rng), distr.sample(&mut rng)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    fn get_hashes(&self, square: Square) -> [u64; N] {
        let mut hashes = [0; N];

        for (i, hash) in hashes.iter_mut().enumerate() {
            *hash = self.hashers[i].hash(square).0;
        }

        hashes
    }
}

fn find_equal_squares(input: &[Vec<u32>]) -> Option<(Square, Square)> {
    let n = input.len();
    let m = input[0].len();

    // < 2^32-1 in order to not overflow multiplication
    const PRIME: u64 = 1_000_000_007;
    const HASHES: usize = 2;

    let hash_combinator = HashCombinator::<PRIME, HASHES>::new(input);

    let min_size = n.min(m);

    let mut min = 0;
    let mut mid = min_size;
    let mut last_mid = 0;
    let mut max = min_size;

    let mut best = None;

    let mut squares: HashMap<_, Vec<Square>> = HashMap::new();

    'search: while mid != last_mid && (max + min) / 2 != 0 {
        squares.clear();
        last_mid = mid;
        mid = (max + min) / 2;

        for y in mid - 1..n {
            for x in mid - 1..m {
                let current = Square { x, y, size: mid };
                let hashes = hash_combinator.get_hashes(current);

                match squares.entry(hashes) {
                    Entry::Occupied(mut entry) => {
                        let equal_square = entry
                            .get()
                            .iter()
                            .find(|&&square| Square::equal(input, square, current));

                        if let Some(&equal_square) = equal_square {
                            best = Some((equal_square, current));
                            min = mid;
                            continue 'search;
                        } else {
                            entry.get_mut().push(current);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![current]);
                    }
                }
            }
        }

        max = mid;
    }

    best
}

fn main() {
    #[rustfmt::skip]
    let input: Vec<Vec<u8>> = vec![vec![1,   2,    3, 4],
                                   vec![5,   6,    7, 8],
                                   vec![9,  10,    1, 2],
                                   vec![13, 14,    5, 6]];

    let input: Vec<Vec<u32>> =
        input.into_iter().map(|x| x.into_iter().map(|x| x as u32).collect()).collect();

    println!("{:?}", find_equal_squares(&input));
}
