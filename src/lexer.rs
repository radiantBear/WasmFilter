use std::collections::LinkedList;
use std::fmt::Debug;
use std::iter::Peekable;
use std::str::Chars;
use wasm_bindgen::prelude::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Comparator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String)
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum JoinType {
    Or,
    And,
    Xor,
    // Pipe
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Name(String),
    Comparator(Comparator),
    Value(Value),
    JoinType(JoinType),
    OpenParen,
    CloseParen
}

#[derive(Debug, PartialEq)]
pub struct TokenData {
    pub token: Token,
    pub source: String,
    pub start: usize,       // 0-indexed, inclusive
    pub start_line: usize,  // 0-indexed, inclusive
    pub start_col: usize,   // 0-indexed, inclusive
    pub end: usize,         // 0-indexed, not inclusive
    pub end_line: usize,    // 0-indexed, inclusive
    pub end_col: usize,     // 0-indexed, not inclusive
}

impl TokenData {
    pub fn to_bare(&self) -> BareTokenData {
        match &self.token {
            Token::Name(_) =>
                BareTokenData{ token: BareToken::Name, start: self.start, start_line: self.start_line, start_col: self.start_col, end: self.end, end_line: self.end_line, end_col: self.end_col },

            Token::Comparator(_) =>
                BareTokenData{ token: BareToken::Comparator, start: self.start, start_line: self.start_line, start_col: self.start_col, end: self.end, end_line: self.end_line, end_col: self.end_col },

            Token::Value(_) =>
                BareTokenData{ token: BareToken::Value, start: self.start, start_line: self.start_line, start_col: self.start_col, end: self.end, end_line: self.end_line, end_col: self.end_col },

            Token::JoinType(_) =>
                BareTokenData{ token: BareToken::JoinType, start: self.start, start_line: self.start_line, start_col: self.start_col, end: self.end, end_line: self.end_line, end_col: self.end_col },

            _ =>
                BareTokenData{ token: BareToken::Paren, start: self.start, start_line: self.start_line, start_col: self.start_col, end: self.end, end_line: self.end_line, end_col: self.end_col }

        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BareToken {
    Name,
    Comparator,
    Value,
    JoinType,
    Paren
}

#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq)]
pub struct BareTokenData {
    pub token: BareToken,
    pub start: usize,       // 0-indexed, inclusive
    pub start_line: usize,  // 0-indexed, inclusive
    pub start_col: usize,   // 0-indexed, inclusive
    pub end: usize,         // 0-indexed, not inclusive
    pub end_line: usize,    // 0-indexed, inclusive
    pub end_col: usize,     // 0-indexed, not inclusive
}

pub fn lex(s: String) -> LinkedList<TokenData> {
    let mut tokens = LinkedList::new();
    let mut cursor = 0usize;
    let mut line = 0usize;
    let mut col = 0usize;

    let s = s.chars();
    let mut s = s.peekable();
    while let Some(c) = s.next() {
        match c {
            '"' => tokens.push_back(lex_string(&mut s, &mut cursor, &mut line, &mut col)),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push_back(lex_name(c, &mut s, &mut cursor, line, &mut col)),
            '0'..='9' | '-' => tokens.push_back(lex_number(c, &mut s, &mut cursor, line, &mut col)),
            '<' | '>' | '=' | '!' => tokens.push_back(lex_comparator(c, &mut s, &mut cursor, line, &mut col)),
            '(' => tokens.push_back(TokenData{
                token: Token::OpenParen,
                source: "(".to_string(),
                start: cursor,
                start_line: line,
                start_col: col,
                end: cursor + 1,
                end_line: line,
                end_col: col + 1
            }),
            ')' => tokens.push_back(TokenData {
                token: Token::CloseParen,
                source: ")".to_string(),
                start: cursor,
                start_line: line,
                start_col: col,
                end: cursor + 1,
                end_line: line,
                end_col: col + 1
            }),
            '|' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start: cursor,
                start_line: line,
                start_col: col,
                end: cursor + 1,
                end_line: line,
                end_col: col + 1
            }),
            '&' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::And),
                source: "&".to_string(),
                start: cursor,
                start_line: line,
                start_col: col,
                end: cursor + 1,
                end_line: line,
                end_col: col + 1
            }),
            '^' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::Xor),
                source: "^".to_string(),
                start: cursor,
                start_line: line,
                start_col: col,
                end: cursor + 1,
                end_line: line,
                end_col: col + 1
            }),
            '\n' => { line += 1; col = 0; cursor += 1; continue },
            c if c.is_whitespace() => { },
            c @ _ => panic!("Unexpected character {}", c)
        }

        col += 1;
        cursor += 1;
    }

    tokens
}

pub fn lex_name(c: char, s: &mut Peekable<Chars>, cursor: &mut usize, line: usize, col: &mut usize) -> TokenData {
    let start = *cursor;
    let start_col = *col;
    let mut name = String::from(c);

    while let Some(c) = s.peek() {
        if !c.is_alphanumeric() && *c != '_' {
            break;
        }

        name.push(*c);
        s.next();
        *col += 1;
        *cursor += 1;
    }

    TokenData {
        source: name.clone(),
        token: Token::Name(name),
        start,
        start_line: line,
        start_col,
        end: *cursor + 1,
        end_line: line,
        end_col: *col + 1
    }
}

pub fn lex_string(s: &mut Peekable<Chars>, cursor: &mut usize, line: &mut usize, col: &mut usize) -> TokenData {
    let start = *cursor;
    let start_line = *line;
    let start_col = *col;
    let mut value = String::new();

    while let Some(c) = s.next() {
        *col += 1;
        *cursor += 1;

        if c == '"' {
            break;
        }
        else if c == '\n' {
            *line += 1;
            *col = 0;
        }

        value.push(c);
    }

    TokenData {
        source: format!("\"{}\"", value),
        token: Token::Value(Value::String(value)),
        start,
        start_line,
        start_col,
        end: *cursor + 1,
        end_line: *line,
        end_col: *col + 1
    }
}

pub fn lex_number(c: char, s: &mut Peekable<Chars>, cursor: &mut usize, line: usize, col: &mut usize) -> TokenData {
    let found_decimal = false;
    let start = *cursor;
    let start_col = *col;
    let mut number_string = String::from(c);
    let mut raw_string = String::from(c);

    while let Some(c) = s.peek() {
        if !c.is_numeric() && *c != ',' && *c != '.' {
            break;
        }
        if *c == '.' && found_decimal {
            panic!("Unexpected second decimal place");
        }

        // Allow commas for splitting large numbers, but not actually part of number
        if *c != ',' {
            number_string.push(*c);
        }
        raw_string.push(*c);
        s.next();
        *col += 1;
        *cursor += 1;
    }

    TokenData {
        source: raw_string,
        token: Token::Value(Value::Number(number_string.parse::<f64>().unwrap())),
        start,
        start_line: line,
        start_col,
        end: *cursor + 1,
        end_line: line,
        end_col: *col + 1
    }
}

pub fn lex_comparator(c: char, s: &mut Peekable<Chars>, cursor: &mut usize, line: usize, col: &mut usize) -> TokenData {
    match c {
        '>' => match s.peek() {
            Some('=') => {
                s.next();
                *col += 1;
                *cursor += 1;
                TokenData {
                    token: Token::Comparator(Comparator::GreaterThanOrEqual),
                    source: ">=".to_string(),
                    start: *cursor - 1,
                    start_line: line,
                    start_col: *col - 1,
                    end: *cursor + 1,
                    end_line: line,
                    end_col: *col + 1
                }
            },
            _ => TokenData {
                token: Token::Comparator(Comparator::GreaterThan),
                source: ">".to_string(),
                start: *cursor,
                start_line: line,
                start_col: *col,
                end: *cursor + 1,
                end_line: line,
                end_col: *col + 1
            }
        },
        '<' => match s.peek() {
            Some('=') => {
                s.next();
                *col += 1;
                *cursor += 1;
                TokenData {
                    token: Token::Comparator(Comparator::LessThanOrEqual),
                    source: "<=".to_string(),
                    start: *cursor - 1,
                    start_line: line,
                    start_col: *col - 1,
                    end: *cursor + 1,
                    end_line: line,
                    end_col: *col + 1
                }
            },
            _ => TokenData {
                token: Token::Comparator(Comparator::LessThan),
                source: "<".to_string(),
                start: *cursor,
                start_line: line,
                start_col: *col,
                end: *cursor + 1,
                end_line: line,
                end_col: *col + 1
            }
        },
        '=' => TokenData {
            token: Token::Comparator(Comparator::Equal),
            source: "=".to_string(),
            start: *cursor,
            start_line: line,
            start_col: *col,
            end: *cursor + 1,
            end_line: line,
            end_col: *col + 1
        },
        '!' => match s.peek() {
            Some('=') => {
                s.next();
                *col += 1;
                *cursor += 1;
                TokenData {
                    token: Token::Comparator(Comparator::NotEqual),
                    source: "!=".to_string(),
                    start: *cursor - 1,
                    start_line: line,
                    start_col: *col - 1,
                    end: *cursor + 1,
                    end_line: line,
                    end_col: *col + 1
                }
            },
            None => panic!("Unexpected end of filter after '='"),
            Some(c) => panic!("Unexpected character '{}' (expected `=` to make `!=`)", c)
        },
        _ => panic!("Passed invalid character `{}` to lex_comparator()", c)
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    pub fn lexes_equal_comparator() {
        let input = "=".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::Equal),
            source: "=".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_not_equal_comparator() {
        let input = "!=".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::NotEqual),
            source: "!=".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 2,
            end_line: 0,
            end_col: 2
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_less_than_comparator() {
        let input = "<".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::LessThan),
            source: "<".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_less_than_or_equal_comparator() {
        let input = "<=".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::LessThanOrEqual),
            source: "<=".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 2,
            end_line: 0,
            end_col: 2
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_greater_than_comparator() {
        let input = ">".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::GreaterThan),
            source: ">".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_greater_than_or_equal_comparator() {
        let input = ">=".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Comparator(Comparator::GreaterThanOrEqual),
            source: ">=".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 2,
            end_line: 0,
            end_col: 2
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_and_join_type() {
        let input = "&".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::JoinType(JoinType::And),
            source: "&".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_or_join_type() {
        let input = "|".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::JoinType(JoinType::Or),
            source: "|".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_xor_join_type() {
        let input = "^".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::JoinType(JoinType::Xor),
            source: "^".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_name() {
        let input = "test".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Name("test".to_string()),
            source: "test".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 4,
            end_line: 0,
            end_col: 4
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_string() {
        let input = "\"test\"".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Value(Value::String("test".to_string())),
            source: "\"test\"".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 6,
            end_line: 0,
            end_col: 6
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_positive_integer() {
        let input = "109".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Value(Value::Number(109.)),
            source: "109".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 3,
            end_line: 0,
            end_col: 3
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }
    
    #[test]
    pub fn lexes_positive_real_number() {
        let input = "109.55".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Value(Value::Number(109.55)),
            source: "109.55".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 6,
            end_line: 0,
            end_col: 6
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }
    #[test]
    pub fn lexes_positive_comma_separated_real_number() {
        let input = "62,109.55".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::Value(Value::Number(62_109.55)),
            source: "62,109.55".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 9,
            end_line: 0,
            end_col: 9
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_open_parentheses() {
        let input = "(".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::OpenParen,
            source: "(".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_close_parentheses() {
        let input = ")".to_string();

        let expected = LinkedList::from([TokenData {
            token: Token::CloseParen,
            source: ")".to_string(),
            start: 0,
            start_line: 0,
            start_col: 0,
            end: 1,
            end_line: 0,
            end_col: 1
        }]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_comparison() {
        let input = "test = \"test\"".to_string();

        let expected = LinkedList::from([
            TokenData {
                token: Token::Name("test".to_string()),
                source: "test".to_string(),
                start: 0,
                start_line: 0,
                start_col: 0,
                end: 4,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start: 5,
                start_line: 0,
                start_col: 5,
                end: 6,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value(Value::String("test".to_string())),
                source: "\"test\"".to_string(),
                start: 7,
                start_line: 0,
                start_col: 7,
                end: 13,
                end_line: 0,
                end_col: 13
            },
        ]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_comparison_without_spaces() {
        let input = "test=\"test\"".to_string();

        let expected = LinkedList::from([
            TokenData {
                token: Token::Name("test".to_string()),
                source: "test".to_string(),
                start: 0,
                start_line: 0,
                start_col: 0,
                end: 4,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start: 4,
                start_line: 0,
                start_col: 4,
                end: 5,
                end_line: 0,
                end_col: 5
            },
            TokenData {
                token: Token::Value(Value::String("test".to_string())),
                source: "\"test\"".to_string(),
                start: 5,
                start_line: 0,
                start_col: 5,
                end: 11,
                end_line: 0,
                end_col: 11
            },
        ]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_comparison_with_newline() {
        let input = "test =\n\"test\"".to_string();

        let expected = LinkedList::from([
            TokenData {
                token: Token::Name("test".to_string()),
                source: "test".to_string(),
                start: 0,
                start_line: 0,
                start_col: 0,
                end: 4,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start: 5,
                start_line: 0,
                start_col: 5,
                end: 6,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value(Value::String("test".to_string())),
                source: "\"test\"".to_string(),
                start: 7,
                start_line: 1,
                start_col: 0,
                end: 13,
                end_line: 1,
                end_col: 6
            },
        ]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_joined_comparisons() {
        let input = "test = \"test\" | test_2  !=\"test_2\"".to_string();

        let expected = LinkedList::from([
            TokenData {
                token: Token::Name("test".to_string()),
                source: "test".to_string(),
                start: 0,
                start_line: 0,
                start_col: 0,
                end: 4,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start: 5,
                start_line: 0,
                start_col: 5,
                end: 6,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value(Value::String("test".to_string())),
                source: "\"test\"".to_string(),
                start: 7,
                start_line: 0,
                start_col: 7,
                end: 13,
                end_line: 0,
                end_col: 13
            },
            TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start: 14,
                start_line: 0,
                start_col: 14,
                end: 15,
                end_line: 0,
                end_col: 15
            },
            TokenData {
                token: Token::Name("test_2".to_string()),
                source: "test_2".to_string(),
                start: 16,
                start_line: 0,
                start_col: 16,
                end: 22,
                end_line: 0,
                end_col: 22
            },
            TokenData {
                token: Token::Comparator(Comparator::NotEqual),
                source: "!=".to_string(),
                start: 24,
                start_line: 0,
                start_col: 24,
                end: 26,
                end_line: 0,
                end_col: 26
            },
            TokenData {
                token: Token::Value(Value::String("test_2".to_string())),
                source: "\"test_2\"".to_string(),
                start: 26,
                start_line: 0,
                start_col: 26,
                end: 34,
                end_line: 0,
                end_col: 34
            },
        ]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    pub fn lexes_joined_comparisons_with_newline() {
        let input = "test = \"test\"\n| test_2  !=\"test_2\"".to_string();

        let expected = LinkedList::from([
            TokenData {
                token: Token::Name("test".to_string()),
                source: "test".to_string(),
                start: 0,
                start_line: 0,
                start_col: 0,
                end: 4,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start: 5,
                start_line: 0,
                start_col: 5,
                end: 6,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value(Value::String("test".to_string())),
                source: "\"test\"".to_string(),
                start: 7,
                start_line: 0,
                start_col: 7,
                end: 13,
                end_line: 0,
                end_col: 13
            },
            TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start: 14,
                start_line: 1,
                start_col: 0,
                end: 15,
                end_line: 1,
                end_col: 1
            },
            TokenData {
                token: Token::Name("test_2".to_string()),
                source: "test_2".to_string(),
                start: 16,
                start_line: 1,
                start_col: 2,
                end: 22,
                end_line: 1,
                end_col: 8
            },
            TokenData {
                token: Token::Comparator(Comparator::NotEqual),
                source: "!=".to_string(),
                start: 24,
                start_line: 1,
                start_col: 10,
                end: 26,
                end_line: 1,
                end_col: 12
            },
            TokenData {
                token: Token::Value(Value::String("test_2".to_string())),
                source: "\"test_2\"".to_string(),
                start: 26,
                start_line: 1,
                start_col: 12,
                end: 34,
                end_line: 1,
                end_col: 20
            },
        ]);
        let result = lex(input);

        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "Unexpected character")]
    pub fn panics_on_unexpected_character() {
        let input = "test @ \"test\"".to_string();

        lex(input);
    }

    #[test]
    #[should_panic(expected = "character '")]
    pub fn panics_on_incomplete_not_equal() {
        let input = "test ! \"test\"".to_string();

        lex(input);
    }

    #[test]
    #[should_panic(expected = "Unexpected end")]
    pub fn panics_on_incomplete_not_equal_2() {
        let input = "test !".to_string();

        lex(input);
    }
}