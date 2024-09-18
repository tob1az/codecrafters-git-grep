use std::env;
use std::io;
use std::process;

#[derive(Debug, Clone)]
enum Matcher {
    StartOfLine,
    EndOfLine,
    WordChar,
    Digit,
    // TODO: &str
    PositiveCharGroup(String),
    NegativeCharGroup(String),
    Literal(char),
    OneOrMore(Vec<Matcher>),
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
                (matches!(c, 'a'..'z') || matches!(c, 'A'..'Z') || c == '_').then_some(1)
            }
            Self::Digit => matches!(c, '0'..'9').then_some(1),
            Self::PositiveCharGroup(g) => g.contains(c).then_some(1),
            Self::NegativeCharGroup(g) => (!g.contains(c)).then_some(1),
            Self::Literal(l) => (*l == c).then_some(1),
            Self::OneOrMore(group) => Self::match_sequence(group, string),
        }
    }

    fn try_parse(pattern: &str, previous: &[Matcher]) -> Option<(Self, usize)> {
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
        } else if pattern.starts_with("+") {
            if !previous.is_empty() {
                Some((Self::OneOrMore(previous.to_vec()), 1))
            } else {
                None
            }
        } else {
            Some((Self::Literal(pattern.chars().next().unwrap()), 1))
        }
    }

    fn match_sequence(matchers: &[Matcher], string: &str) -> Option<usize> {
        let mut match_count = 0;
        'exit: loop {
            let mut increment = 0;
            for m in matchers {
                let remainder = &string[match_count + increment..];
                if let Some(parsed) = m.match_some(remainder) {
                    increment += parsed;
                } else {
                    break 'exit;
                }
            }
            match_count += increment;
        }
        if match_count > 0 {
            Some(match_count)
        } else {
            None
        }
    }
}

struct Expression {
    matchers: Vec<Matcher>,
    start_of_line: bool,
    end_of_line: bool,
    min_length: usize,
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

    fn min_len(&self) -> usize {
        self.min_length
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
        let mut min_length = 0;
        while pattern_index < value.len() {
            let remainder = &value[pattern_index..];
            let previous = if matchers.is_empty() {
                &[]
            } else {
                let last = matchers.len() - 1;
                &matchers[last..=last]
            };
            match Matcher::try_parse(remainder, previous) {
                Some((Matcher::StartOfLine, offset)) => {
                    start_of_line = true;
                    pattern_index += offset;
                }
                Some((Matcher::EndOfLine, offset)) => {
                    end_of_line = true;
                    pattern_index += offset;
                }
                Some((matcher @ Matcher::OneOrMore(_), offset)) => {
                    // TODO: support group
                    // TODO: pass previous as &mut to avoid copies
                    matchers.pop();
                    matchers.push(matcher);
                    pattern_index += offset;
                }
                Some((matcher, offset)) => {
                    min_length += 1;
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
            min_length,
        })
    }
}

fn match_pattern(input_line: &str, expression: &Expression) -> bool {
    if input_line.len() < expression.min_len() {
        return false;
    }
    let mut input_index = 0;
    while input_index <= input_line.len() - expression.min_len() {
        let remainder = &input_line[input_index..];
        if expression.match_str(remainder) {
            return if expression.till_end_of_string() {
                remainder.len() == expression.min_len()
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
