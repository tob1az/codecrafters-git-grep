use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else if pattern == "\\d" {
        input_line.chars().any(|c| matches!(c, '0'..'9'))
    } else if pattern == "\\w" {
        input_line
            .chars()
            .all(|c| matches!(c, 'a'..'z') || matches!(c, 'A'..'Z') || c == '_')
    } else if pattern.len() > 2 && pattern.starts_with('[') && pattern.ends_with(']') {
        let positive_group = &pattern[1..pattern.len() - 1];
        input_line.chars().any(|c| positive_group.contains(c))
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
