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

#[derive(Clone,Copy)]
pub enum Case {
	Sensitive,
	Insensitive
}

/// Counts the frequencies of chars from a string with as many threads as cpu's.
///
/// Identical to character_frequencies() but allows custom setting
/// of Case::Sensitive or Case::Insensitive. 
/// With Sensitive, a and A will be counted differently
/// With Insensitive, a and A will be counted as the same character 
pub fn character_frequencies_with_case(text: &str,case:Case) -> HashMap<char, usize> {
    character_frequencies_with_n_threads_with_case(text, num_cpus::get(),case)
}

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

/// Counts the frequencies of chars from a string with the ammount of threads specified. By default this is case insensitive
/// By default this is Case-insensitive
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
	character_frequencies_with_n_threads_with_case(text, threads,Case::Insensitive)
}

/// Counts the frequencies of chars from a string with the ammount of threads specified.
///
/// Identical to character_frequencies_with_n_threads() but allows custom setting
/// of Case::Sensitive or Case::Insensitive. 
/// With Sensitive, a and A will be counted differently
/// With Insensitive, a and A will be counted as the same character 
pub fn character_frequencies_with_n_threads_with_case(text: &str, threads: usize,case:Case) -> HashMap<char, usize> {
    if threads <= 1 {
        return sequential_character_frequencies(text,case);
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
		cases:Case
    ) {
        let tx = tx.clone();
        let shared = shared.clone();
        thread::spawn(move || {
            let frequency_map =
                character_frequencies_range(shared.as_str(), from, from + chunk_size - 1,cases);
            tx.send(frequency_map).unwrap();
        });
    }

    let mut from = 0;
    for _ in 0..threads_with_less_data {
        generate_counting_thread(from, chunk_size, &tx, &shared,case);
        from += chunk_size;
    }
    for _ in 0..threads_with_more_data {
        generate_counting_thread(from, chunk_size + 1, &tx, &shared,case);
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

pub fn sequential_character_frequencies(text: &str,case:Case) -> HashMap<char, usize> {
    character_frequencies_range(text, 0, text.len() - 1,case)
}

fn character_frequencies_range(text: &str, from: usize, to: usize, case:Case) -> HashMap<char, usize> {
    let mut frequency_map: HashMap<char, usize> = HashMap::new();
    for character in text.chars().skip(from).take(to - from + 1) {
		let c = match case {
			Case::Insensitive => character.to_ascii_lowercase(),
			_=> character,
		};
        *frequency_map
            .entry(c)
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

    #[test]
    fn test_unicode() {
        let result = character_frequencies_range("维维尼熊aabbbccd|@", 0, 13,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('b', 3);
        expected.insert('维',2);
        expected.insert('尼',1);
        expected.insert('熊',1);
        expected.insert('a', 2);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_full_with_case() {
        let result = character_frequencies_range("XXaabbbccd|@", 0, 11,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('b', 3);
        expected.insert('X', 2);
        expected.insert('a', 2);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_full() {
        let result = character_frequencies_range("aaaabbbccd|@", 0, 11,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_consecutive_left() {
        let result = character_frequencies_range("aaaa", 0, 2,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 3);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_consecutive_right() {
        let result = character_frequencies_range("aaaa", 1, 3,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 3);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_consecutive_center() {
        let result = character_frequencies_range("aaaa", 1, 2,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 2);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_consecutive_whole() {
        let result = character_frequencies_range("aaaa", 0, 3,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_only_one_left() {
        let result = character_frequencies_range("aaa", 0, 0,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_only_one_right() {
        let result = character_frequencies_range("aaa", 2, 2,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_character_frequencies_range_only_one_center() {
        let result = character_frequencies_range("aaa", 1, 1,Case::Sensitive);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sequential_character_frequencies() {
        let result = character_frequencies("aaaabbbccd|@");
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parallel_character_frequencies_more_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 13);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parallel_character_frequencies_less_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 5);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parallel_character_frequencies_single_thread() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 1);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parallel_character_frequencies_prime_threads() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 7);
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parallel_character_frequencies_n_threads() {
        let result = character_frequencies("aaaabbbccd|@");
        let mut expected: HashMap<char, usize> = HashMap::new();
        expected.insert('a', 4);
        expected.insert('b', 3);
        expected.insert('c', 2);
        expected.insert('d', 1);
        expected.insert('|', 1);
        expected.insert('@', 1);

        assert_eq!(result, expected);
    }
}
