use std::env;
use std::io;
use std::process;

enum Matcher {
    WordChar,
    Digit,
    PositiveCharGroup(String),
    NegativeCharGroup(String),
    Literal(char),
}

impl Matcher {
    fn match_some(&self, string: &str) -> Option<usize> {
        if string.is_empty() {
            return None;
        }
        let c = string.chars().next().unwrap();
        if match self {
            Self::WordChar => matches!(c, 'a'..'z') || matches!(c, 'A'..'Z') || c == '_',
            Self::Digit => matches!(c, '0'..'9'),
            Self::PositiveCharGroup(g) => g.contains(c),
            Self::NegativeCharGroup(g) => !g.contains(c),
            Self::Literal(l) => *l == c,
        } {
            Some(1)
        } else {
            None
        }
    }

    fn try_parse(pattern: &str) -> Option<(Self, usize)> {
        if pattern.is_empty() {
            return None;
        }
        if pattern.starts_with("\\d") {
            Some((Self::Digit, 2))
        } else if pattern.starts_with("\\w") {
            Some((Self::WordChar, 2))
        } else if pattern.starts_with("[^") {
            if let Some(end) = pattern.find(']') {
                Some((Self::NegativeCharGroup(pattern[2..end].to_owned()), end + 1))
            } else {
                None
            }
        } else if pattern.starts_with("[") {
            if let Some(end) = pattern.find(']') {
                Some((Self::PositiveCharGroup(pattern[1..end].to_owned()), end + 1))
            } else {
                None
            }
        } else {
            Some((Self::Literal(pattern.chars().next().unwrap()), 1))
        }
    }
}

struct Expression {
    matchers: Vec<Matcher>,
}

impl Expression {
    fn match_str(&self, input: &str) -> bool {
        let mut offset = 0;
        for m in &self.matchers {
            if offset >= input.len() {
                return false;
            }
            let remaining_input = &input[offset..];
            if let Some(shift) = m.match_some(remaining_input) {
                offset += shift;
            } else {
                return false;
            }
        }
        true
    }

    fn len(&self) -> usize {
        self.matchers.len()
    }
}

impl TryFrom<&str> for Expression {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut pattern_index = 0;
        let mut matchers = Vec::new();
        while pattern_index < value.len() {
            let remainder = &value[pattern_index..];
            if let Some((matcher, offset)) = Matcher::try_parse(remainder) {
                matchers.push(matcher);
                pattern_index += offset;
            } else {
                return Err(());
            }
        }
        Ok(Self { matchers })
    }
}

fn match_pattern(input_line: &str, expression: &Expression) -> bool {
    if input_line.len() < expression.len() {
        return false;
    }
    let mut input_index = 0;
    while input_index <= input_line.len() - expression.len() {
        let remainder = &input_line[input_index..];
        if expression.match_str(remainder) {
            return true;
        } else {
            input_index += 1;
        }
    }
    false
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

    if let Ok(expression) = Expression::try_from(pattern.as_ref()) {
        if match_pattern(&input_line, &expression) {
            process::exit(0)
        } else {
            process::exit(1)
        }
    } else {
        process::exit(1)
    }
}
