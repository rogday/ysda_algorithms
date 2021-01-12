use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use std::convert::TryInto;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::collections::HashMap;

fn get_table(filename: &str) -> Option<HashMap<u32, f64>> {
    let file = BufReader::new(File::open(filename).ok()?);
    let mut ret = HashMap::with_capacity(500_000);

    for line in file.lines() {
        let line = line.ok()?;
        let mut it = line.split(' ');

        let mut quadgram: [u8; 4] = it.next()?.as_bytes().try_into().ok()?;
        quadgram.iter_mut().for_each(|x| *x -= b'A');
        let quadgram: u32 = u32::from_be_bytes(quadgram);

        let occurrences: u64 = it.next()?.parse().ok()?;

        ret.insert(quadgram, occurrences);
    }

    let total = ret.iter().map(|(_, v)| v).sum::<u64>() as f64;

    Some(ret.into_iter().map(|(k, v)| (k, (v as f64 / total).ln())).collect())
}

fn prepare_file(filename: &str) -> Option<(String, Vec<u8>)> {
    let mut file = File::open(filename).ok()?;

    let mut original = String::new();
    file.read_to_string(&mut original).ok()?;

    let filtered: Vec<u8> = original
        .bytes()
        .filter(|&ch| (ch as char).is_ascii_alphabetic())
        .map(|ch| (ch as char).to_ascii_uppercase() as u8 - b'A')
        .collect();

    Some((original, filtered))
}

fn perturbate_new(key: &[u8; 26], prng: &mut SmallRng) -> [u8; 26] {
    let mut ret = *key;

    loop {
        let a_k = prng.gen_range(0, 26);
        let b_k = prng.gen_range(0, 26);

        if a_k == b_k {
            continue;
        }

        ret.swap(a_k, b_k);

        break;
    }

    ret
}

fn work(
    filename: &str,
    quadgrams: &HashMap<u32, f64>,
    frequency_table: &[(u8, u32)],
) -> Option<()> {
    let (original, filtered) = prepare_file(filename)?;

    let mut frequencies: [(u32, usize); 26] = [(0, 0); 26];
    for (i, v) in frequencies.iter_mut().enumerate() {
        v.1 = i;
    }

    for &ch in &filtered {
        frequencies[ch as usize].0 += 1;
    }

    frequencies.sort_unstable();
    frequencies.reverse();

    let mut best_key = [0; 26];

    for (&(_, from), &(to, _)) in frequencies.iter().zip(frequency_table) {
        best_key[from as usize] = to - b'A';
    }

    let seed: u64 = rand::thread_rng().gen();
    // let seed = 3779529701869089186;
    let mut prng: SmallRng = SeedableRng::seed_from_u64(seed);
    println!("Using seed: {}", seed);

    let mut best_score: f64 = f64::MIN;

    for _ in 0..10_000 {
        let new_key = perturbate_new(&best_key, &mut prng);

        let mut score = 0.0;

        let mut quadgram: [u8; 4] = filtered[0..4].try_into().ok()?;
        quadgram.iter_mut().for_each(|q| *q = new_key[*q as usize]);
        let mut quadgram: u32 = u32::from_be_bytes(quadgram);

        for &byte in filtered.iter().skip(4) {
            quadgram = (quadgram << 8) | new_key[byte as usize] as u32;
            score += quadgrams.get(&quadgram).unwrap_or(&-20.);
        }

        if score > best_score {
            best_score = score;
            best_key = new_key;

            // println!("{}", best_score);
        }
    }

    // fix this bs
    println!(
        "{}",
        original
            .chars()
            .map(|x| {
                if !x.is_ascii_alphabetic() {
                    return x;
                }

                let index = (x.to_ascii_uppercase() as u8 - b'A') as usize;
                let ret = best_key.get(index).map(|ch| (ch + b'A') as char).unwrap_or(x);

                if x.is_ascii_uppercase() {
                    ret
                } else {
                    ret.to_ascii_lowercase()
                }
            })
            .collect::<String>()
    );

    Some(())
}

fn main() {
    let frequency_table = [
        (b'E', 21912),
        (b'T', 16587),
        (b'A', 14810),
        (b'O', 14003),
        (b'I', 13318),
        (b'N', 12666),
        (b'S', 11450),
        (b'R', 10977),
        (b'H', 10795),
        (b'D', 7874),
        (b'L', 7253),
        (b'U', 5246),
        (b'C', 4943),
        (b'M', 4761),
        (b'F', 4200),
        (b'Y', 3853),
        (b'W', 3819),
        (b'G', 3693),
        (b'P', 3316),
        (b'B', 2715),
        (b'V', 2019),
        (b'K', 1257),
        (b'X', 315),
        (b'Q', 205),
        (b'J', 188),
        (b'Z', 128),
    ];

    let quadgrams = get_table("decoder/input/english_quadgrams.txt").unwrap();

    // abcdefghijklmnopqrstuvwxyz
    // fbngurdsxvptzhaqilwyjekocm
    work("decoder/input/one.txt", &quadgrams, &frequency_table).unwrap();
    work("decoder/input/two.txt", &quadgrams, &frequency_table).unwrap();
}
