use character_frequency::*;

fn main() {
    let frequency_map = character_frequencies("Hello, World!");

    println!("Character frequencies:");
    for (character, frequency) in frequency_map {
        println!("\'{}\': {}", character, frequency);
    }

    let frequency_map = character_frequencies_w_case("Hello, WORLD", CaseSense::Sensitive);

    println!("Character frequencies (case sensitive):");
    for (character, frequency) in frequency_map {
        println!("\'{}\': {}", character, frequency);
    }
}
