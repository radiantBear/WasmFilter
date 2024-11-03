use std::collections::LinkedList;
use std::fmt::Debug;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq)]
pub enum Comparator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum JoinType {
    Or,
    And,
    Xor,
    // Pipe
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Name(String),
    Comparator(Comparator),
    Value(String),
    JoinType(JoinType),
    OpenParen,
    CloseParen
}

pub fn lex(s: String) -> LinkedList<Token> {
    let mut tokens = LinkedList::new();

    let mut s = s.chars();
    while let Some(c) = s.next() {
        match c {
            '(' => tokens.push_back(Token::OpenParen),
            ')' => tokens.push_back(Token::CloseParen),
            '"' => tokens.push_back(Token::Value(lex_value(&mut s))),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push_back(Token::Name(lex_name(c, &mut s))),
            '<' | '>' | '=' | '!' => tokens.push_back(Token::Comparator(lex_comparator(c, &mut s))),
            '|' => tokens.push_back(Token::JoinType(JoinType::Or)),
            '&' => tokens.push_back(Token::JoinType(JoinType::And)),
            '^' => tokens.push_back(Token::JoinType(JoinType::Xor)),
            c if c.is_whitespace() => continue,
            c @ _ => panic!("Unexpected character {}", c)
        }
    }

    tokens
}

pub fn lex_name(c: char, s: &mut Chars) -> String {
    let mut s = s.peekable();
    let mut name = String::from(c);

    while let Some(c) = s.peek() {
        if !c.is_alphanumeric() && *c != '_' {
            break;
        }

        name.push(*c);
        s.next();
    }

    name
}

pub fn lex_value(s: &mut Chars) -> String {
    let mut value = String::new();

    while let Some(c) = s.next() {
        if c == '"' {
            break;
        }

        value.push(c);
    }

    value
}

pub fn lex_comparator(c: char, s: &mut Chars) -> Comparator {
    let mut s = s.peekable();

    match c {
        '>' => match s.peek() {
            Some('=') => { s.next(); Comparator::GreaterThanOrEqual },
            _ => Comparator::GreaterThan
        },
        '<' => match s.peek() {
            Some('=') => { s.next(); Comparator::LessThanOrEqual },
            _ => Comparator::LessThan
        },
        '=' => Comparator::Equal,
        '!' => match s.peek() {
            Some('=') => { s.next(); Comparator::NotEqual },
            None => panic!("Unexpected end of filter after '='"),
            Some(c) => panic!("Unexpected character '{}'", c)
        },
        _ => panic!("Unexpected char {}", c)
    }
}