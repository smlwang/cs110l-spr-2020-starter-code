/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut ret = v.clone();
    for value in ret.iter_mut() {
        *value += n;
    }
    ret
}

fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for value in v.iter_mut() {
        *value += n;
    }
}

fn dedup(v: &mut Vec<i32>) {
    let mut unique = HashSet::<i32>::new();
    let mut uni_number = 0;
    let size = v.len();
    for i in 0..size {
        if unique.insert(v[i]) {
            v[uni_number] = v[i];
            uni_number += 1
        }
    }
    v.truncate(uni_number)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
