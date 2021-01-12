use std::{iter::FromIterator, num::NonZeroUsize};

use itertools::Itertools;

fn decomposition(n: usize, mut k: usize) -> Vec<usize> {
    let chunk_size = (n as f64).sqrt().round() as usize;

    let mut prisoners: Vec<(usize, Vec<Option<NonZeroUsize>>)> = (1..=n)
        .map(NonZeroUsize::new)
        .chunks(chunk_size)
        .into_iter()
        .map(FromIterator::from_iter)
        .map(|chunk: Vec<_>| (chunk.len(), chunk))
        .collect();

    k -= 1;
    let mut index = 0;
    let mut ret = vec![];

    for killed in 0..n {
        let mut greedy_index = 0;
        index = (index + k) % (n - killed);

        let (size, chunk) = prisoners
            .iter_mut()
            .find(|&&mut (i, _)| {
                greedy_index += i;
                greedy_index > index
            })
            .unwrap();

        *size -= 1;
        let number = chunk
            .iter_mut()
            .filter(|x| std::matches!(x, Some(_)))
            .rfind(|_| {
                greedy_index -= 1;
                greedy_index == index
            })
            .unwrap()
            .take()
            .unwrap();

        ret.push(number.get());
    }

    ret
}

fn segment_tree(n: usize, mut k: usize) -> Vec<usize> {
    let mut tree = vec![0; 4 * n];
    let mut ret = vec![];

    k -= 1;
    let mut index = 0;
    for killed in 0..n {
        index = (index + k) % (n - killed);

        let mut pos = 0;
        let mut number = 0;

        let mut last_size;
        let mut size = n;

        let mut elements = 0;
        while size != 1 {
            last_size = size;
            size = (size + 1) >> 1;

            let left = pos * 2 + 1;
            let right = left + 1;

            let left_part = size - tree[left];
            if elements + left_part > index {
                pos = left;
                tree[pos] += 1;
            } else {
                pos = right;

                elements += left_part;
                number += size;
                size = size.min(last_size - size);
            }
        }

        ret.push(number + 1);
    }

    ret
}

fn main() {
    let n = 100_000;
    let k = 100_000;

    let dec = decomposition(n, k);
    let seg = segment_tree(n, k);

    if dec != seg {
        println!("dec: {:?}\nseg: {:?}", dec, seg);
    } else {
        println!("OK")
    }
}
