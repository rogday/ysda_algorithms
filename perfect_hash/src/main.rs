use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

use std::time::{Instant};

struct HashSet<V> {
    array:  Vec<Option<V>>,
    hasher: Hasher,
}

struct PerfectHashSet {
    inner: HashSet<HashSet<u32>>,
}

struct Hasher {
    a: u64,
    b: u64,
    m: usize,
}

impl Hasher {
    fn new(m: usize) -> Self {
        let mut rng = rand::thread_rng();

        let da = Uniform::new_inclusive(1, std::u32::MAX);
        let db = Uniform::new_inclusive(0, std::u32::MAX);

        let a: u32 = da.sample(&mut rng);
        let b: u32 = db.sample(&mut rng);

        Self { a: a as u64, b: b as u64, m }
    }

    fn hash(&self, x: u32) -> usize {
        const P: u64 = 4_294_967_311;
        (self.a.wrapping_mul(x as u64).wrapping_add(self.b) % P) as usize % self.m
    }
}

impl PerfectHashSet {
    fn gen_bucket(bucket: &[u32]) -> HashSet<u32> {
        let m = bucket.len() * bucket.len();
        let mut init: Vec<Option<u32>> = vec![None; m];

        let hasher = 'gen_bucket: loop {
            let hasher = Hasher::new(m);

            for &x in bucket.iter() {
                let hash = hasher.hash(x);

                if init[hash].is_some() {
                    init.iter_mut().for_each(|x| *x = None);
                    continue 'gen_bucket;
                }

                init[hash] = Some(x);
            }
            break hasher;
        };

        HashSet { array: init, hasher }
    }

    fn new(input: &[u32]) -> Self {
        let capacity = input.len();
        let mut init = vec![vec![]; capacity];

        loop {
            let hasher = Hasher::new(capacity);

            for &x in input {
                init[hasher.hash(x)].push(x);
            }

            let sum: usize = init.iter().map(|x| x.len() * x.len()).sum();

            if sum >= 4 * input.len() {
                init.iter_mut().for_each(|x| *x = vec![]);
                continue;
            }

            let mut ret = PerfectHashSet {
                inner: HashSet::<HashSet<u32>> {
                    array: (0..capacity).map(|_| None).collect(),
                    hasher,
                },
            };

            for (i, bucket) in init.iter().enumerate().filter(|(_, b)| !b.is_empty()) {
                ret.inner.array[i] = Some(PerfectHashSet::gen_bucket(bucket));
            }

            break ret;
        }
    }

    fn contains_helper(&self, v: u32) -> Option<bool> {
        let h1 = self.inner.hasher.hash(v);
        let h2 = self.inner.array[h1].as_ref()?.hasher.hash(v);
        self.inner.array[h1].as_ref()?.array[h2].map(|x| x == v)
    }

    fn contains(&self, v: u32) -> bool {
        self.contains_helper(v).unwrap_or(false)
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let m: usize = 100_000;
    let max = 2_000_000_000;
    let n = 1;

    for i in 0..n {
        println!("{}", i);

        let v1 = {
            let mut tmp: Vec<u32> = (0..m).map(|_| rng.gen_range(0, max)).collect();
            tmp.sort_unstable();
            tmp.dedup();
            tmp
        };

        let now = Instant::now();

        let phs = PerfectHashSet::new(&v1);
        assert!(v1.iter().map(|&x| phs.contains(x)).all(|x| x));

        println!("{}", now.elapsed().as_micros());
    }
}
