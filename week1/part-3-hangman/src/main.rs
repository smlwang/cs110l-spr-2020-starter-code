// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn read_guess_ch() -> char {
    let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("Error reading line.");
    guess.chars().nth(0).unwrap()
}

fn get_prompt_word(guess_flag: &Vec<bool>, secret_word_chars: &Vec<char>) -> String {
    let word_length = secret_word_chars.len();
    let mut prompt_word = String::new();
    for i in 0..word_length {
        prompt_word.push(if !guess_flag[i] {
            '-'
        } else {
            secret_word_chars[i]
        });
    }
    prompt_word
}

fn guess_check(guess_flag: &mut Vec<bool>, secret_word_chars: &Vec<char>, c: char) -> bool {
    let mut result = false;

    let word_length = secret_word_chars.len();
    for i in 0..word_length {
        if guess_flag[i] {
            continue;
        }
        if secret_word_chars[i] == c {
            result = true;
            guess_flag[i] = true;
        }
    }
    result
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    let word_length = secret_word_chars.len();
    let mut guess_flag = vec![false; word_length];
    let mut have_guessed = String::new();
    let mut left_guess_number = NUM_INCORRECT_GUESSES;
    let check_done = |bools: &Vec<bool>| -> bool {
        for b in bools {
            if !b {
                return false
            }
        }
        true
    };
    loop {
        if check_done(&guess_flag) {
            println!("Congratulations you guessed the secret word: {secret_word}!");
            break;
        }
        if left_guess_number <= 0 {
            println!("Sorry, you ran out of guesses!");
            break;
        }
        let prompt_word = get_prompt_word(&guess_flag, &secret_word_chars);
        println!("The word so far is {prompt_word}");
        println!("You have guessed the following letters: {have_guessed}");
        println!("You have {left_guess_number} guesses left");
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");

        let c = read_guess_ch();
        have_guessed.push(c);
        let success = guess_check(&mut guess_flag, &secret_word_chars, c);

        left_guess_number -= 1;
        if success {
            left_guess_number += 1
        } else {
            println!("Sorry, that letter is not in the word");
        }
        println!()
    }
}
#[test]
fn foo() {
    let mut s = String::from("hello");
    let ref1 = &s;
    let ref2 = &ref1;
    let ref3 = &ref2;
    s = String::from("goodbye");
    println!("{}", ref3.to_uppercase());
}

fn drip_drop() -> &String {
    let s = String::from("hello world!");
    &s
}

#[test]
fn bar() {
    let s1 = String::from("hello");
    let mut v = Vec::new();
    v.push(s1);
    let s2: String = v[0].clone();
    println!("{}", s2);
}

fn pass_char(c : char) {
    let mut s1 = String::from("hello");
    let s2 = &s1;
    println!("{c}, {s1}, {s2}");
    s1.push('a');
}
#[test]
fn test_pass_char() {
    let c = 'a';
    pass_char(c);
    assert_eq!(c, 'a');
}