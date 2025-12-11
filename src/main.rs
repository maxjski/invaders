use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    let secret_number = rand::rng().random_range(1..=100);

    println!("The seecret number is {secret_number}");

    println!("Enter and guess a number!");

    let mut guess = String::new();

    io::stdin()
        .read_line(&mut guess)
        .expect("Something went wrong");

    let guess: u32 = guess.trim().parse().expect("Error parsing");

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too little!"),
        Ordering::Equal => println!("Nice, you guessed"),
        Ordering::Greater => println!("Too big!"),
    }
}
