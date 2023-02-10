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

/// CaseSense enables counting characters in a Case Sensitive way.
/// * InsensitiveASCIIOnly - ignores case, but only for ASCII characters,
/// 'A' and 'a' are counted as the same but Greek letter 'Σ' is
/// counted as different from it's lowercase version 'σ' because it's not ASCII.
/// All ascii characters get converted to lowercase before counting.
/// InsensitiveASCIIOnly is the default.
/// * Insensitive - ignores case based on Unicode Derived Core
/// Property Lowercase, so 'A'=='a' and also 'Σ'=='σ'.
/// This does not deal with situations where case depends on position within
/// a word. It changes all UTF8 characters to lowercase one at a time.
/// Some UTF8 characters have a lowercase version that is a string, if that
/// happens the code will panic!() if Insensitive is the CaseSense.
/// * Sensitive - Each character is counted separately.
/// 'A' != 'a' and 'Σ'!='σ'. No characters are changed to lowercase.
/// * See also <https://doc.rust-lang.org/std/string/struct.String.html#method.to_ascii_lowercase>
#[derive(Clone, Copy)]
pub enum CaseSense {
    Insensitive,
    InsensitiveASCIIOnly,
    Sensitive,
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

/// same as character_frequences() but with Case Sensitivity
pub fn character_frequencies_w_case(text: &str, case: CaseSense) -> HashMap<char, usize> {
    character_frequencies_with_n_threads_w_case(text, num_cpus::get(), case)
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
    character_frequencies_with_n_threads_w_case(text, threads, CaseSense::InsensitiveASCIIOnly)
}

/// same as character_frequencies_with_n_threads(), with Case Sensitivity
pub fn character_frequencies_with_n_threads_w_case(
    text: &str,
    threads: usize,
    case: CaseSense,
) -> HashMap<char, usize> {
    if threads <= 1 {
        return sequential_character_frequencies_w_case(text, case);
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
        case: CaseSense,
    ) {
        let tx = tx.clone();
        let shared = shared.clone();
        thread::spawn(move || {
            let frequency_map =
                character_frequencies_range(shared.as_str(), from, from + chunk_size - 1, case);
            tx.send(frequency_map).unwrap();
        });
    }

    let mut from = 0;
    for _ in 0..threads_with_less_data {
        generate_counting_thread(from, chunk_size, &tx, &shared, case);
        from += chunk_size;
    }
    for _ in 0..threads_with_more_data {
        generate_counting_thread(from, chunk_size + 1, &tx, &shared, case);
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
    character_frequencies_range(text, 0, text.len() - 1, CaseSense::InsensitiveASCIIOnly)
}

// same as sequuential_character_frequencies but with Case Sensitivity
pub fn sequential_character_frequencies_w_case(
    text: &str,
    case: CaseSense,
) -> HashMap<char, usize> {
    character_frequencies_range(text, 0, text.len() - 1, case)
}

fn character_frequencies_range(
    text: &str,
    from: usize,
    to: usize,
    case_sense: CaseSense,
) -> HashMap<char, usize> {
    let mut frequency_map: HashMap<char, usize> = HashMap::new();
    for character in text.chars()
        .skip(from)
        .take(to - from + 1)
        .map(|ch|  match case_sense {
            CaseSense::Insensitive => match ch.to_lowercase().len() {
                1 => ch.to_lowercase().next().unwrap(),
       	        _ => panic!("Unicode character {:?} {} when converted to lowercase is a multicharacter String not a character", ch, ch ),},
            CaseSense::InsensitiveASCIIOnly => ch.to_ascii_lowercase(),
            CaseSense::Sensitive=> ch,})
        {
            *frequency_map.entry(character).or_insert(0) += 1;
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
    fn expected_freq(s: &str) -> HashMap<char, usize> {
        HashMap::<char, usize>::from_iter(s.split(" ").map(|chunk| {
            (
                chunk.chars().next().unwrap(),
                usize::from_str_radix(&chunk.chars().skip(1).collect::<String>(), 10).unwrap(),
            )
        }))
    }

    #[test]
    fn test_character_frequencies_range_full() {
        let result =
            character_frequencies_range("aaaabbbccd|@", 0, 11, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_left() {
        let result = character_frequencies_range("aaaa", 0, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a3"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_right() {
        let result = character_frequencies_range("aaaa", 1, 3, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a3"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_center() {
        let result = character_frequencies_range("aaaa", 1, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a2"));
        let result = character_frequencies_range("baab", 1, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a2"));
        let result = character_frequencies_range("bacb", 1, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a1 c1"));
        let result = character_frequencies_range("dcab", 1, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a1 c1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_whole() {
        let result = character_frequencies_range("aaaa", 0, 3, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a4"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_left() {
        let result = character_frequencies_range("aaa", 0, 0, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_right() {
        let result = character_frequencies_range("aaa", 2, 2, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_center() {
        let result = character_frequencies_range("aaa", 1, 1, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(result, expected_freq("a1"));
    }

    #[test]
    fn test_sequential_character_frequencies() {
        let result = character_frequencies("aaaabbbccd|@");
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_more_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 13);
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_less_threads_than_characters() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 5);
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_single_thread() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 1);
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_prime_threads() {
        let result = character_frequencies_with_n_threads("aaaabbbccd|@", 7);
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_n_threads() {
        let result = character_frequencies("aaaabbbccd|@");
        assert_eq!(result, expected_freq("a4 b3 c2 d1 |1 @1"));
    }

    #[test]
    fn test_character_frequencies_range_full_w_case() {
        let result = character_frequencies_range("AaaaBbBCCd|@", 0, 11, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a3 b1 C2 d1 |1 @1 A1 B2"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_left_w_case() {
        let result = character_frequencies_range("aaaA", 0, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a3"));
        let result = character_frequencies_range("Aaaa", 0, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2 A1"));
        let result = character_frequencies_range("AaAa", 0, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1 A2"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_right_w_case() {
        let result = character_frequencies_range("Aaaa", 1, 3, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a3"));
        let result = character_frequencies_range("AaAa", 1, 3, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2 A1"));
        let result = character_frequencies_range("AaaA", 1, 3, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2 A1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_center_w_case() {
        let result = character_frequencies_range("aaaa", 1, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2"));
        let result = character_frequencies_range("baAb", 1, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1 A1"));
        let result = character_frequencies_range("bAcb", 1, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1 c1"));
        let result = character_frequencies_range("dcab", 1, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1 c1"));
    }

    #[test]
    fn test_character_frequencies_range_consecutive_whole_w_case() {
        let result = character_frequencies_range("aaaa", 0, 3, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a4"));
        let result = character_frequencies_range("aAaa", 0, 3, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1 a3"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_left_w_case() {
        let result = character_frequencies_range("aaa", 0, 0, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1"));
        let result = character_frequencies_range("AaA", 0, 0, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_right_w_case() {
        let result = character_frequencies_range("aaa", 2, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1"));
        let result = character_frequencies_range("BaA", 2, 2, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1"));
    }

    #[test]
    fn test_character_frequencies_range_only_one_center_w_case() {
        let result = character_frequencies_range("aaa", 1, 1, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a1"));
        let result = character_frequencies_range("aAa", 1, 1, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1"));
    }

    #[test]
    fn test_sequential_character_frequencies_w_case() {
        let result = character_frequencies_w_case("AaabbbccdEEE|@", CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1 a2 b3 c2 d1 |1 @1 E3"));
    }

    #[test]
    fn test_parallel_character_frequencies_more_threads_than_characters_w_case() {
        let result =
            character_frequencies_with_n_threads_w_case("AabbbccdE|@", 13, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1 a1 b3 c2 d1 |1 @1 E1"));
    }

    #[test]
    fn test_parallel_character_frequencies_less_threads_than_characters_w_case() {
        let result =
            character_frequencies_with_n_threads_w_case("AaaabbbccdEEE|@", 5, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("A1 a3 b3 c2 d1 |1 @1 E3"));
    }

    #[test]
    fn test_parallel_character_frequencies_single_thread_w_case() {
        let result =
            character_frequencies_with_n_threads_w_case("aaaAbbBccd|@", 1, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a3 b2 B1 A1 c2 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_prime_threads_w_case() {
        let result =
            character_frequencies_with_n_threads_w_case("AaaBbbCcd|@", 7, CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2 A1 B1 b2 c1 C1 d1 |1 @1"));
    }

    #[test]
    fn test_parallel_character_frequencies_n_threads_w_case() {
        let result = character_frequencies_w_case("AaaBbbCcd|@", CaseSense::Sensitive);
        assert_eq!(result, expected_freq("a2 A1 B1 b2 c1 C1 d1 |1 @1"));
    }

    #[test]
    fn test_unicode_case_sensitive() {
        let greek_upper = "ὈΔΥΣΣΕΎΣ";
        let greek_lower = "ὀδυσσεύς";
        let greek_mix = "ὀδυσσεύςὈΔΥΣΣΕΎΣ";
        let resultu = character_frequencies_w_case(greek_upper, CaseSense::Sensitive);
        let resultl = character_frequencies_w_case(greek_lower, CaseSense::Sensitive);
        let resultm = character_frequencies_w_case(greek_mix, CaseSense::Sensitive);
        assert_eq!(resultu, expected_freq("Ὀ1 Δ1 Υ1 Σ3 Ε1 Ύ1"));
        assert_eq!(resultl, expected_freq("ὀ1 δ1 υ1 σ2 ε1 ς1 ύ1"));
        assert_eq!(
            resultm,
            expected_freq("Ὀ1 Δ1 Υ1 Σ3 Ε1 Ύ1 ὀ1 δ1 υ1 σ2 ε1 ς1 ύ1")
        );
    }

    #[test]
    fn test_unicode_case_insensitiveasciionly() {
        let greek_upper = "ὈΔΥΣΣΕΎΣ";
        let greek_lower = "ὀδυσσεύς";
        let greek_mix = "ὀδυσσεύςὈΔΥΣΣΕΎΣ";
        let resultu = character_frequencies_w_case(greek_upper, CaseSense::InsensitiveASCIIOnly);
        let resultl = character_frequencies_w_case(greek_lower, CaseSense::InsensitiveASCIIOnly);
        let resultm = character_frequencies_w_case(greek_mix, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(resultu, expected_freq("Ὀ1 Δ1 Υ1 Σ3 Ε1 Ύ1"));
        assert_eq!(resultl, expected_freq("ὀ1 δ1 υ1 σ2 ε1 ς1 ύ1"));
        assert_eq!(
            resultm,
            expected_freq("Ὀ1 Δ1 Υ1 Σ3 Ε1 Ύ1 ὀ1 δ1 υ1 σ2 ε1 ς1 ύ1")
        );
    }

    #[test]
    fn test_unicode_case_insensitive() {
        let greek_upper = "ὈΔΥΣΣΕΎΣ";
        let greek_lower = "ὀδυσσεύς";
        let greek_mix = "ὀδυσσεύςὈΔΥΣΣΕΎΣ";
        let resultu = character_frequencies_w_case(greek_upper, CaseSense::Insensitive);
        let resultl = character_frequencies_w_case(greek_lower, CaseSense::Insensitive);
        let resultm = character_frequencies_w_case(greek_mix, CaseSense::Insensitive);
        assert_eq!(resultu, expected_freq("ὀ1 δ1 υ1 σ3 ε1 ύ1"));
        assert_eq!(resultl, expected_freq("ὀ1 δ1 υ1 σ2 ε1 ς1 ύ1"));
        assert_eq!(resultm, expected_freq("ὀ2 δ2 υ2 σ5 ε2 ς1 ύ2"));
    }

    #[test]
    fn test_unicode_case_irrelevant() {
        let chinese = "夫物芸芸，各復歸其根，歸根曰靜";
        let expect = expected_freq("夫1 物1 芸2 ，2 各1 復1 其1 歸2 根2 曰1 靜1");
        let resultc_s = character_frequencies_w_case(chinese, CaseSense::Sensitive);
        let resultc_ia = character_frequencies_w_case(chinese, CaseSense::Insensitive);
        let resultc_i = character_frequencies_w_case(chinese, CaseSense::InsensitiveASCIIOnly);
        assert_eq!(resultc_s, expect);
        assert_eq!(resultc_ia, expect);
        assert_eq!(resultc_i, expect);
    }
}
