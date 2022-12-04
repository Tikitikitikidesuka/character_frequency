# Character Frequency

A Rust library for counting character frequencies over multiple threads

## Functions

- `character_frequencies(text: &str) -> HashMap<char, usize>`
Returns a map with the frequencies counted on the text parameter.
It will run on as many threads as cpu's are available. 
- `character_frequencies_with_n_threads(text: &str, threads: usize) -> HashMap<char, usize>`:
Returns a map with the frequencies counted on the text parameter.
It will run on the specified ammount of threads.
- `character_frequencies_with_case(text: &str,case:Case) -> HashMap<char, usize>`
- `character_frequencies_with_n_threads_with_case(text: &str, threads: usize,case:Case) -> HashMap<char, usize>`:
Identical to above but with case sensitivity turned on (so 'A' will be counted separate from 'a')

## Example
This example counts the character frequencies of `Hello, World!` and print them afterwards:
```rust
use character_frequency::*;

let frequency_map = character_frequencies("Hello, World!");

println!("Character frequencies:");
for (character, frequency) in frequency_map {
    println!("\'{}\': {}", character, frequency);
}

let frequency_map2 = character_frequencies("Hello, World!",Case::Sensitive);

```
