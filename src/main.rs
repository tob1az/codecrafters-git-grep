use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut pattern_index = 0;
    for c in input_line.chars() {
        if pattern_index >= pattern.len() {
            return true;
        }
        let remaining_pattern = &pattern[pattern_index..];
        if remaining_pattern.starts_with("\\d") {
            if matches!(c, '0'..'9') {
                pattern_index += 2;
            } else {
                pattern_index = 0;
            }
        } else if remaining_pattern.starts_with("\\w") {
            if matches!(c, 'a'..'z') || matches!(c, 'A'..'Z') || c == '_' {
                pattern_index += 2;
            } else {
                pattern_index = 0;
            }
        } else if remaining_pattern.starts_with("[^") {
            if let Some(end) = remaining_pattern.find(']') {
                let negative_group = &remaining_pattern[pattern_index + 2..end];
                if !negative_group.contains(c) {
                    pattern_index += end + 1;
                } else {
                    pattern_index = 0;
                }
            } else {
                pattern_index = 0;
            }
        } else if remaining_pattern.starts_with("[") {
            if let Some(end) = remaining_pattern.find(']') {
                let positive_group = &remaining_pattern[pattern_index + 1..end];
                if positive_group.contains(c) {
                    pattern_index += end + 1;
                } else {
                    pattern_index = 0;
                }
            }
        } else {
            if pattern[pattern_index..=pattern_index].contains(c) {
                pattern_index += 1;
            } else {
                pattern_index = 0;
            }
        }
    }
    pattern_index >= pattern.len()
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
