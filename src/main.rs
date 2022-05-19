use std::cmp::max;
use std::time::Instant;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Sender;
use std::thread;

fn main() {
    let file_name = "text.txt";

    let contents = fs::read_to_string(file_name)
        .expect("Something went wrong reading the file");

    let start1 = Instant::now();
    let frequencies1 = character_frequencies(&contents);
    let duration1 = start1.elapsed();

    let start2 = Instant::now();
    let frequencies2 = parallel_character_frequencies(&contents, num_cpus::get());
    let duration2 = start2.elapsed();

    assert_eq!(frequencies1, frequencies2);
    println!("Sequential time: {:?}", duration1);
    println!("Parallel time: {:?}", duration2);
}

fn character_frequencies(text: &str) -> HashMap<char, usize> {
    character_frequencies_range(text, 0, text.len() - 1)
}

fn character_frequencies_range(text: &str, from: usize, to: usize) -> HashMap<char, usize> {
    let mut frequency_map: HashMap<char, usize> = HashMap::new();
    for character in text.chars().skip(from).take(to - from + 1) {
        *frequency_map.entry(character.to_ascii_lowercase()).or_insert(0) += 1;
    }
    frequency_map
}

fn parallel_character_frequencies(text: &str, mut threads: usize) -> HashMap<char, usize> {
    if threads < 1 {
        threads = 1;
    }

    let (tx, rx) = mpsc::channel::<HashMap<char, usize>>();

    let shared = Arc::new(String::from(text));
    let chunk_size = max(1, text.len() / threads);

    let threads_with_more_data = text.len() % threads;
    let threads_with_less_data = threads - threads_with_more_data;

    fn generate_thread(from: usize, chunk_size: usize, tx: &Sender<HashMap<char, usize>>, shared: &Arc<String>) {
        let tx = tx.clone();
        let shared = shared.clone();
        thread::spawn(move || {
            let frequency_map = character_frequencies_range(
                shared.as_str(),
                from,
                from + chunk_size - 1
            );
            tx.send(frequency_map).unwrap();
        });
    }

    let mut from = 0;
    for _ in 0..threads_with_less_data {
        generate_thread(from, chunk_size, &tx, &shared);
        from += chunk_size;
    }
    for _ in 0..threads_with_more_data {
        generate_thread(from, chunk_size + 1, &tx, &shared);
        from += chunk_size + 1;
    }

    let mut out = rx.recv().unwrap();
    for _ in 0..threads-1 {
        out = add_frequencies(out, rx.recv().unwrap());
    }
    out
}

fn add_frequencies(a: HashMap<char, usize>, b: HashMap<char, usize>) -> HashMap<char, usize> {
    let mut out = a;
    for (character, frequency) in b {
        *out.entry(character).or_insert(0) += frequency;
    }
    out
}


#[test]
fn test_character_frequencies_range_full() {
    let result = character_frequencies_range("aaaabbbccd|@", 0, 11);
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
    let result = character_frequencies_range("aaaa", 0, 2);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 3);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_consecutive_right() {
    let result = character_frequencies_range("aaaa", 1, 3);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 3);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_consecutive_center() {
    let result = character_frequencies_range("aaaa", 1, 2);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 2);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_consecutive_whole() {
    let result = character_frequencies_range("aaaa", 0, 3);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 4);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_only_one_left() {
    let result = character_frequencies_range("aaa", 0, 0);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 1);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_only_one_right() {
    let result = character_frequencies_range("aaa", 2, 2);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 1);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies_range_only_one_center() {
    let result = character_frequencies_range("aaa", 1, 1);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 1);

    assert_eq!(result, expected);
}

#[test]
fn test_character_frequencies() {
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
fn test_parallel_character_frequencies() {
    let result = parallel_character_frequencies("aaaabbbccd|@", 10);
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
    let result = parallel_character_frequencies("aaaabbbccd|@", 13);
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
    let result = parallel_character_frequencies("aaaabbbccd|@", 5);
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
    let result = parallel_character_frequencies("aaaabbbccd|@", 1);
    let mut expected: HashMap<char, usize> = HashMap::new();
    expected.insert('a', 4);
    expected.insert('b', 3);
    expected.insert('c', 2);
    expected.insert('d', 1);
    expected.insert('|', 1);
    expected.insert('@', 1);

    assert_eq!(result, expected);
}







