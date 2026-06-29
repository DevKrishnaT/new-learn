use std::io;

use chrono::{Datelike, Local};

fn main() {
    let mut input = String::new();
    let mut num_input = String::new();

    println!("Enter your name:");
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read name");

    println!("Enter your age:");
    io::stdin()
        .read_line(&mut num_input)
        .expect("Failed to read age");

    let age: u32 = num_input.trim().parse().expect("That's not a valid age.");

    if age > 100 {
        println!("Wow! Try entering a human age.");
        return;
    }

    let current_year = Local::now().year();
    let year_turns_100 = current_year + (100 - age) as i32;

    println!(
        "{} will turn 100 in the year {}",
        input.trim(),
        year_turns_100
    );
}
