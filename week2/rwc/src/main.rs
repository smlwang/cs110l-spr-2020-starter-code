use std::env;
use std::process;
use std::io::{self, BufRead};
use std::fs::File;

fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    let file = File::open(filename)?;
    let mut lines = Vec::<String>::new();
    for line in io::BufReader::new(file).lines() {
        lines.push(line?);
    }
    return Ok(lines);
}

fn count_words(lines: &Vec<String>) -> usize {
    let mut words = 0;
    for line in lines.clone().iter_mut() {
        for _ in line.split_whitespace() {
            words += 1
        }
    }
    words
}
fn count_lines(lines: &Vec<String>) -> usize {
   lines.len()
}
fn count_chars(lines: &Vec<String>) -> usize {
    let mut chars = 0;
    for line in lines.iter() {
        chars += line.as_bytes().len();
    }
    chars
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];

    // Your code here :)
    let lines = read_file_lines(filename).unwrap();

    let words = count_words(&lines);
    let chars = count_chars(&lines);
    let line = count_lines(&lines);
    println!("word: {words}");
    println!("char: {chars}");
    println!("line: {line}");
}
