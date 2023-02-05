//! # About
//! Multithreaded character frequency counter.
//!
//! Counts the character frequencies in a text over multiple threads.
//!

use std::cmp::max;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread;

/// Counts the frequencies of chars from a string with as many threads as cpu's.
///
/// # Examples
/// ```
/// use character_frequency::*;
/// # use std::collections::HashMap;
///
/// let frequency_map = character_frequencies("Hello, World!");
///
/// println!("Character frequencies:");
/// for (character, frequency) in frequency_map {
///     println!("\'{}\': {}", character, frequency);
/// }
///
/// # let mut expected: HashMap<char, usize> = HashMap::new();
/// # expected.insert('h', 1);
/// # expected.insert('e', 1);
/// # expected.insert('l', 3);
/// # expected.insert('o', 2);
/// # expected.insert('w', 1);
/// # expected.insert('r', 1);
/// # expected.insert('d', 1);
/// # expected.insert('!', 1);
/// # expected.insert(',', 1);
/// # expected.insert(' ', 1);
/// ```
pub fn character_frequencies(text: &str) -> HashMap<char, usize> {
    character_frequencies_with_n_threads(text, num_cpus::get())
}

/// Counts the frequencies of chars from a string with the amount of threads specified.
///
/// # Examples
/// ```
/// use character_frequency::*;
/// # use std::collections::HashMap;
///
/// let frequency_map = character_frequencies_with_n_threads("Hello, World!", 8);
///
/// println!("Character frequencies:");
/// for (character, frequency) in frequency_map {
///     println!("\'{}\': {}", character, frequency);
/// }
///
/// # let mut expected: HashMap<char, usize> = HashMap::new();
/// # expected.insert('h', 1);
/// # expected.insert('e', 1);
/// # expected.insert('l', 3);
/// # expected.insert('o', 2);
/// # expected.insert('w', 1);
/// # expected.insert('r', 1);
/// # expected.insert('d', 1);
/// # expected.insert('!', 1);
/// # expected.insert(',', 1);
/// # expected.insert(' ', 1);
/// ```
pub fn character_frequencies_with_n_threads(text: &str, threads: usize) -> HashMap<char, usize> {
    if threads <= 1 {
        return sequential_character_frequencies(text);
    }

    let (tx, rx) = mpsc::channel::<HashMap<char, usize>>();

    let shared = Arc::new(String::from(text));
    let chunk_size = max(1, text.len() / threads);

    let threads_with_more_data = text.len() % threads;
    let threads_with_less_data = threads - threads_with_more_data;

    fn generate_counting_thread(
        from: usize,
        chunk_size: usize,
        tx: &Sender<HashMap<char, usize>>,
        shared: &Arc<String>,
    ) {
        let tx = tx.clone();
        let shared = shared.clone();
        thread::spawn(move || {
            let frequency_map =
                character_frequencies_range(shared.as_str(), from, from + chunk_size - 1);
            tx.send(frequency_map).unwrap();
        });
    }

    let mut from = 0;
    for _ in 0..threads_with_less_data {
        generate_counting_thread(from, chunk_size, &tx, &shared);
        from += chunk_size;
    }
    for _ in 0..threads_with_more_data {
        generate_counting_thread(from, chunk_size + 1, &tx, &shared);
        from += chunk_size + 1;
    }

    fn generate_adding_thread(
        a: HashMap<char, usize>,
        b: HashMap<char, usize>,
        tx: &Sender<HashMap<char, usize>>,
    ) {
        let tx = tx.clone();
        thread::spawn(move || {
            let sum = add_frequencies(a, b);
            tx.send(sum).unwrap();
        });
    }

    let mut waiting_num: usize = threads;
    let mut received = Vec::with_capacity(2);
    while waiting_num > 0 {
        received.push(rx.recv().unwrap());
        waiting_num -= 1;

        if received.len() >= 2 {
            generate_adding_thread(
                received.pop().unwrap(),
                received.pop().unwrap(),
                &tx.clone(),
            );
            waiting_num += 1;
        }
    }
    received.pop().unwrap()
}

pub fn sequential_character_frequencies(text: &str) -> HashMap<char, usize> {
    character_frequencies_range(text, 0, text.len() - 1)
}

fn character_frequencies_range(text: &str, from: usize, to: usize) -> HashMap<char, usize> {
    let mut frequency_map: HashMap<char, usize> = HashMap::new();
    for character in text.chars().skip(from).take(to - from + 1) {
        *frequency_map
            .entry(character.to_ascii_lowercase())
            .or_insert(0) += 1;
    }
    frequency_map
}

fn add_frequencies(a: HashMap<char, usize>, b: HashMap<char, usize>) -> HashMap<char, usize> {
    let mut out = a;
    for (character, frequency) in b {
        *out.entry(character).or_insert(0) += frequency;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // convenience function for testing
    // given "a4 b3 c2 d1 e1", return hashmap {a:4, b:3, c:2, d;1, e:1}
    fn hashfreq(s: &str) -> HashMap<char, usize> {
        let mut hm: HashMap<char, usize> = HashMap::new();
        for i in s.split(" ") {
            let ch = i.chars().next().unwrap();
            let num = usize::from_str_radix(i.get(1..).unwrap(), 10).unwrap();
            hm.insert(ch, num);
        }
        hm
    }

    #[test]
    fn test_character_frequencies_range_full() {
        let result = character_frequencies_range("aaaabbbccd|@", 0, 11);
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_left() {
        let result = character_frequencies_range("aaaa", 0, 2);
        assert_eq!(result, hashfreq("a3"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_right() {
        let result = character_frequencies_range("aaaa", 1, 3);
        assert_eq!(result, hashfreq("a3"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_center() {
        let result = character_frequencies_range("aaaa", 1, 2);
        assert_eq!(result, hashfreq("a2"));
        let result = character_frequencies_range("baab", 1, 2);
        assert_eq!(result, hashfreq("a2"));
        let result = character_frequencies_range("bacb", 1, 2);
        assert_eq!(result, hashfreq("a1 c1"));
        let result = character_frequencies_range("dcab", 1, 2);
        assert_eq!(result, hashfreq("a1 c1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_whole() {
        let result = character_frequencies_range("aaaa", 0, 3);
        assert_eq!(result, hashfreq("a4"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_left() {
        let result = character_frequencies_range("aaa", 0, 0);
        assert_eq!(result, hashfreq("a1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_right() {
        let result = character_frequencies_range("aaa", 2, 2);
        assert_eq!(result, hashfreq("a1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_center() {
        let result = character_frequencies_range("aaa", 1, 1);
        assert_eq!(result, hashfreq("a1"));
    }

    #[test]
    fn test_sequential_character_frequencies() {
        let result = character_frequencies("aaaabbbccd|@");
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_more_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 13);
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_less_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 5);
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_single_thread() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 1);
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_prime_threads() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 7);
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_n_threads() {
        let result = character_frequencies("aaaabbbccd|@");
        assert_eq!(result, hashfreq("a4 b3 c2 d1 |1 @1"));
    }
}
