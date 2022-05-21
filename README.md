# Character Frequency

A Rust library for counting character frequencies over multiple threads

## Usage

Add this to your `Cargo.toml`:
[dependencies]
character_frequency = "0.1.0"

## Functions

- `character_frequencies(text: &str) -> HashMap<char, usize>`:
Returns a map with the frequencies counted on the text parameter.
It will run on as many threads as cpu's are available. 
- `character_frequencies_with_n_threads(text: &str, threads: usize) -> HashMap<char, usize>`:
Returns a map with the frequencies counted on the text parameter.
It will run on the specified ammount of threads.

## Example
This example counts the character frequencies of `Hello, World!` and print them afterwards:
```rust
use character_frequency::*;
use std::collections::HashMap;

let frequency_map = character_frequencies("Hello, World!");

println!("Character frequencies:");
for (character, frequency) in frequency_map {
    println!("{}: {}", character, frequency);
}
```