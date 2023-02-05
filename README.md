# Character Frequency

A Rust library for counting character frequencies over multiple threads

## Functions

- `character_frequencies(text: &str) -> HashMap<char, usize>`
Returns a map with the frequencies counted on the text parameter.
It will run on as many threads as cpu's are available. 
- `character_frequencies_with_n_threads(text: &str, threads: usize) -> HashMap<char, usize>`:
Returns a map with the frequencies counted on the text parameter.
It will run on the specified ammount of threads.
- `character_frequencies_w_case(text: &str,case:CaseSense) -> HashMap<char, usize>`
Same as character_frequencies() but with Case Sensitive counting
- `character_frequencies_with_n_threads_w_case(text: &str,case:CaseSense) -> HashMap<char, usize>`
Same as character_frequencies_with_n_threads() but with Case Sensitive counting

- `CaseSense::InsensitiveASCIIOnly` - Converts ASCII characters to lowercase before counting
 This is the default.
- `CaseSense::Insensitive` - Converts all UTF8 characters to lowercase before counting.  If
 the character's lowercase version is a string not a character, it panics. 
- `CaseSense::Sensitive` - Doesn't convert any characters to lowercase before counting. 

## Example
This example counts the character frequencies of `Hello, World!` and print them afterwards:

```rust
use character_frequency::*;

let frequency_map = character_frequencies("Hello, World!");

println!("Character frequencies:");
for (character, frequency) in frequency_map {
    print!("\'{}\': {}", character, frequency);
}
//Character frequencies:
//'r': 1 'd': 1 'o': 2 '!': 1 ',': 1 ' ': 1 'e': 1 'h': 1 'w': 1 'l': 3

let frequency_map = character_frequencies("Hello WORLD",CaseSense::Sensitive);

println!("Character frequencies:");
for (character, frequency) in frequency_map {
    print!("\'{}\': {}", character, frequency);
}
//Character frequencies:
//'R': 1 'D': 1 'O': 1 'o': 1 ' ': 1 'e': 1 'H': 1 'W': 1 'l': 2 'L': 1

```
