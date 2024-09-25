use std::cell::RefCell;
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
    OneOrMore(Box<Matcher>),
    ZeroOrOne(Box<Matcher>),
    Wildcard,
    GroupStart,
    GroupEnd,
    Alteration,
    Group(Vec<Matcher>, Vec<Matcher>),
    Backreference(usize),
}

impl Matcher {
    fn match_some<'a>(
        &self,
        string: &'a str,
        matched_groups: &RefCell<Vec<&'a str>>,
    ) -> Option<usize> {
        let c = string.chars().next()?;
        match self {
            Self::StartOfLine
            | Self::EndOfLine
            | Self::GroupStart
            | Self::GroupEnd
            | Self::Alteration => Some(0),
            Self::WordChar => {
                (matches!(c, 'a'..='z') || matches!(c, 'A'..='Z') || c == '_').then_some(1)
            }
            Self::Digit => matches!(c, '0'..='9').then_some(1),
            Self::PositiveCharGroup(g) => g.contains(c).then_some(1),
            Self::NegativeCharGroup(g) => (!g.contains(c)).then_some(1),
            Self::Literal(l) => (*l == c).then_some(1),
            Self::OneOrMore(matcher) => Self::match_sequence(matcher, string, matched_groups),
            Self::ZeroOrOne(matcher) => matcher.match_some(string, matched_groups).or(Some(0)),
            Self::Wildcard => Some(1),
            Self::Group(left, right) => Self::match_group(left, string, matched_groups)
                .or_else(|| Self::match_group(right, string, matched_groups)),
            Self::Backreference(n) => string
                .starts_with(matched_groups.borrow()[*n - 1])
                .then(|| matched_groups.borrow()[*n - 1].len()),
        }
    }

    fn parse_backreference(pattern: &str) -> Option<(usize, usize)> {
        if !pattern.starts_with("\\") {
            return None;
        }
        let number_size = pattern
            .chars()
            .skip(1)
            .take_while(|c| c.is_numeric())
            .count();
        if number_size == 0 {
            return None;
        }
        let number = pattern[1..=number_size].parse().ok()?;
        Some((number, number_size + 1))
    }

    fn try_parse(pattern: &str, previous: Option<&Matcher>) -> Option<(Self, usize)> {
        if pattern.starts_with("^") {
            Some((Self::StartOfLine, 1))
        } else if pattern.starts_with("$") {
            Some((Self::EndOfLine, 1))
        } else if pattern.starts_with("\\d") {
            Some((Self::Digit, 2))
        } else if pattern.starts_with("\\w") {
            Some((Self::WordChar, 2))
        } else if let Some((number, length)) = Self::parse_backreference(pattern) {
            Some((Self::Backreference(number), length))
        } else if pattern.starts_with("[^") {
            pattern
                .find(']')
                .map(|end| (Self::NegativeCharGroup(pattern[2..end].to_owned()), end + 1))
        } else if pattern.starts_with("[") {
            pattern
                .find(']')
                .map(|end| (Self::PositiveCharGroup(pattern[1..end].to_owned()), end + 1))
        } else if pattern.starts_with("+") {
            Some((Self::OneOrMore(Box::new(previous?.clone())), 1))
        } else if pattern.starts_with("?") {
            Some((Self::ZeroOrOne(Box::new(previous?.clone())), 1))
        } else if pattern.starts_with(".") {
            Some((Self::Wildcard, 1))
        } else if pattern.starts_with("(") {
            Some((Self::GroupStart, 1))
        } else if pattern.starts_with(")") {
            Some((Self::GroupEnd, 1))
        } else if pattern.starts_with("|") {
            Some((Self::Alteration, 1))
        } else {
            Some((Self::Literal(pattern.chars().next()?), 1))
        }
    }

    fn match_sequence<'a>(
        matcher: &Matcher,
        string: &'a str,
        matched_groups: &RefCell<Vec<&'a str>>,
    ) -> Option<usize> {
        let mut match_count = 0;
        loop {
            let remainder = &string[match_count..];
            if let Some(matched) = matcher.match_some(remainder, matched_groups) {
                match_count += matched;
            } else {
                break;
            }
        }

        if match_count > 0 {
            Some(match_count)
        } else {
            None
        }
    }

    fn match_group<'a>(
        matchers: &[Matcher],
        string: &'a str,
        matched_groups: &RefCell<Vec<&'a str>>,
    ) -> Option<usize> {
        if matchers.is_empty() {
            return None;
        }
        let mut match_len = 0;
        for m in matchers {
            let remainder = &string[match_len..];
            match_len += m.match_some(remainder, matched_groups)?;
        }
        matched_groups.borrow_mut().push(&string[0..match_len]);
        Some(match_len)
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
        let mut matched_groups = RefCell::new(Vec::new());
        for m in &self.matchers {
            if offset >= input.len() {
                return false;
            }
            let remaining_input = &input[offset..];
            if let Some(shift) = m.match_some(remaining_input, &mut matched_groups) {
                offset += shift;
            } else {
                return false;
            }
        }
        if self.till_end_of_string() {
            offset >= input.len()
        } else {
            true
        }
    }

    fn from_start_of_string(&self) -> bool {
        self.start_of_line
    }

    fn till_end_of_string(&self) -> bool {
        self.end_of_line
    }
}

struct Group {
    start_index: usize,
    alternative_index: Option<usize>,
}

impl TryFrom<&str> for Expression {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut pattern_index = 0;
        let mut matchers = Vec::new();
        let mut start_of_line = false;
        let mut end_of_line = false;
        let mut groups = Vec::new();
        while pattern_index < value.len() {
            let remainder = &value[pattern_index..];
            match Matcher::try_parse(remainder, matchers.last()) {
                Some((Matcher::StartOfLine, offset)) => {
                    start_of_line = true;
                    pattern_index += offset;
                }
                Some((Matcher::EndOfLine, offset)) => {
                    end_of_line = true;
                    pattern_index += offset;
                }
                Some((Matcher::GroupStart, offset)) => {
                    groups.push(Group {
                        start_index: matchers.len(),
                        alternative_index: None,
                    });
                    pattern_index += offset;
                }
                Some((Matcher::Alteration, offset)) => {
                    let group = groups
                        .last_mut()
                        .ok_or("Alteration not in group".to_owned())?;

                    if group.alternative_index.is_some() {
                        return Err("Double alteration in group".into());
                    }
                    group.alternative_index = Some(matchers.len());
                    pattern_index += offset;
                }
                Some((Matcher::GroupEnd, offset)) => {
                    let group = groups.pop().ok_or("Stray )".to_owned())?;
                    let alternative_index =
                        group.alternative_index.unwrap_or_else(|| matchers.len());
                    let right = matchers.split_off(alternative_index);
                    let left = matchers.split_off(group.start_index);
                    matchers.push(Matcher::Group(left, right));
                    pattern_index += offset;
                }
                Some((matcher @ Matcher::OneOrMore(_), offset))
                | Some((matcher @ Matcher::ZeroOrOne(_), offset)) => {
                    // TODO: pass previous as &mut to avoid copies
                    matchers.pop();
                    matchers.push(matcher);
                    pattern_index += offset;
                }
                Some((matcher @ Matcher::Backreference(n), offset)) => {
                    if matchers
                        .iter()
                        .filter(|m| matches!(m, Matcher::Group(_, _)))
                        .count()
                        < n
                    {
                        return Err("Invalid back reference".into());
                    }
                    matchers.push(matcher);
                    pattern_index += offset;
                }
                Some((matcher, offset)) => {
                    matchers.push(matcher);
                    pattern_index += offset;
                }
                None => return Err("Failed to parse a matcher".into()),
            }
        }
        if !groups.is_empty() {
            Err("Unclosed group".into())
        } else {
            Ok(Self {
                matchers,
                start_of_line,
                end_of_line,
            })
        }
    }
}

fn match_pattern(input_line: &str, expression: &Expression) -> bool {
    if input_line.is_empty() {
        return false;
    }
    let mut input_index = 0;
    while input_index < input_line.len() {
        let remainder = &input_line[input_index..];
        if expression.match_str(remainder) {
            return true;
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

    match Expression::try_from(pattern.as_ref()) {
        Ok(expression) => {
            if match_pattern(&input_line, &expression) {
                process::exit(0)
            } else {
                process::exit(1)
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1)
        }
    }
}
