use std::env;
use std::io;
use std::process;

#[derive(Debug)]
enum Matcher {
    StartOfLine,
    EndOfLine,
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
        match self {
            Self::StartOfLine | Self::EndOfLine => Some(0),
            Self::WordChar => {
                (matches!(c, 'a'..'z') || matches!(c, 'A'..'Z') || c == '_').then(|| 1)
            }
            Self::Digit => matches!(c, '0'..'9').then(|| 1),
            Self::PositiveCharGroup(g) => g.contains(c).then(|| 1),
            Self::NegativeCharGroup(g) => (!g.contains(c)).then(|| 1),
            Self::Literal(l) => (*l == c).then(|| 1),
        }
    }

    fn try_parse(pattern: &str) -> Option<(Self, usize)> {
        if pattern.is_empty() {
            return None;
        }
        if pattern.starts_with("^") {
            Some((Self::StartOfLine, 1))
        } else if pattern.starts_with("$") {
            Some((Self::EndOfLine, 1))
        } else if pattern.starts_with("\\d") {
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
    start_of_line: bool,
    end_of_line: bool,
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

    fn from_start_of_string(&self) -> bool {
        self.start_of_line
    }

    fn till_end_of_string(&self) -> bool {
        self.end_of_line
    }
}

impl TryFrom<&str> for Expression {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut pattern_index = 0;
        let mut matchers = Vec::new();
        let mut start_of_line = false;
        let mut end_of_line = false;
        while pattern_index < value.len() {
            let remainder = &value[pattern_index..];
            match Matcher::try_parse(remainder) {
                Some((Matcher::StartOfLine, offset)) => {
                    start_of_line = true;
                    pattern_index += offset;
                }
                Some((Matcher::EndOfLine, offset)) => {
                    end_of_line = true;
                    pattern_index += offset;
                }
                Some((matcher, offset)) => {
                    matchers.push(matcher);
                    pattern_index += offset;
                }
                None => return Err(()),
            }
        }
        Ok(Self {
            matchers,
            start_of_line,
            end_of_line,
        })
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
            return if expression.till_end_of_string() {
                remainder.len() == expression.len()
            } else {
                true
            };
        } else if expression.from_start_of_string() {
            return false;
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
