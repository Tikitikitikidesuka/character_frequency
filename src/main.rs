use std::cmp::max;
use std::time::Instant;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Sender;
use std::thread;
mod lib;
use lib::*;

fn main() {
    let file_name = "test.txt";

    let contents = fs::read_to_string(file_name)
        .expect("Something went wrong reading the file");

    // OLD TEST
    /*
    let start1 = Instant::now();
    let frequencies1 = sequential_character_frequencies(&contents);
    let duration1 = start1.elapsed();
    */

    let start2 = Instant::now();
    let frequencies2 = character_frequencies(&contents);
    let duration2 = start2.elapsed();

    //assert_eq!(frequencies1, frequencies2);
    //println!("Sequential time: {:?}", duration1);
    println!("Parallel time: {:?}", duration2);
}





