use std::collections::LinkedList;
use std::fmt::Debug;
use std::iter::Peekable;
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

#[derive(Debug, Eq, PartialEq)]
pub struct TokenData {
    pub token: Token,
    pub source: String,
    pub start_line: usize,  // 0-indexed, inclusive
    pub start_col: usize,   // 0-indexed, inclusive
    pub end_line: usize,    // 0-indexed, inclusive
    pub end_col: usize,     // 0-indexed, not inclusive
}

pub fn lex(s: String) -> LinkedList<TokenData> {
    let mut tokens = LinkedList::new();
    let mut line = 0usize;
    let mut col = 0usize;

    let s = s.chars();
    let mut s = s.peekable();
    while let Some(c) = s.next() {
        match c {
            '"' => tokens.push_back(lex_value(&mut s, &mut line, &mut col)),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push_back(lex_name(c, &mut s, &line, &mut col)),
            '<' | '>' | '=' | '!' => tokens.push_back(lex_comparator(c, &mut s, &line, &mut col)),
            '(' => tokens.push_back(TokenData{
                token: Token::OpenParen,
                source: "(".to_string(),
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1
            }),
            ')' => tokens.push_back(TokenData {
                token: Token::CloseParen,
                source: ")".to_string(),
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1
            }),
            '|' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1
            }),
            '&' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::And),
                source: "&".to_string(),
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1
            }),
            '^' => tokens.push_back(TokenData {
                token: Token::JoinType(JoinType::Xor),
                source: "^".to_string(),
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1
            }),
            '\n' => { line += 1; col = 0; continue },
            c if c.is_whitespace() => { col += 1; continue },
            c @ _ => panic!("Unexpected character {}", c)
        }
        
        col += 1;
    }

    tokens
}

pub fn lex_name(c: char, s: &mut Peekable<Chars>, line: &usize, col: &mut usize) -> TokenData {
    let start_col = *col;
    let mut name = String::from(c);

    while let Some(c) = s.peek() {
        if !c.is_alphanumeric() && *c != '_' {
            break;
        }

        name.push(*c);
        s.next();
        *col += 1;
    }

    TokenData {
        source: name.clone(),
        token: Token::Name(name),
        start_line: *line,
        start_col,
        end_line: *line,
        end_col: *col + 1
    }
    
}

pub fn lex_value(s: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) -> TokenData {
    let start_line = *line;
    let start_col = *col;
    let mut value = String::new();

    while let Some(c) = s.next() {
        *col += 1;
        
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
        token: Token::Value(value),
        start_line,
        start_col,
        end_line: *line,
        end_col: *col + 1
    }
}

pub fn lex_comparator(c: char, s: &mut Peekable<Chars>, line: &usize, col: &mut usize) -> TokenData {
    match c {
        '>' => match s.peek() {
            Some('=') => { 
                s.next();
                *col += 1;
                TokenData { 
                    token: Token::Comparator(Comparator::GreaterThanOrEqual),
                    source: ">=".to_string(),
                    start_line: *line,
                    start_col: *col - 1,
                    end_line: *line,
                    end_col: *col + 1
                }
            },
            _ => TokenData {
                token: Token::Comparator(Comparator::GreaterThan),
                source: ">".to_string(),
                start_line: *line,
                start_col: *col,
                end_line: *line,
                end_col: *col + 1
            }
        },
        '<' => match s.peek() {
            Some('=') => {
                s.next();
                *col += 1;
                TokenData {
                    token: Token::Comparator(Comparator::LessThanOrEqual),
                    source: "<=".to_string(),
                    start_line: *line,
                    start_col: *col - 1,
                    end_line: *line,
                    end_col: *col + 1
                }
            },
            _ => TokenData {
                token: Token::Comparator(Comparator::LessThan),
                source: "<".to_string(),
                start_line: *line,
                start_col: *col,
                end_line: *line,
                end_col: *col + 1
            }
        },
        '=' => TokenData {
            token: Token::Comparator(Comparator::Equal),
            source: "=".to_string(),
            start_line: *line,
            start_col: *col,
            end_line: *line,
            end_col: *col + 1
        },
        '!' => match s.peek() {
            Some('=') => { 
                s.next();
                *col += 1;
                TokenData {
                    token: Token::Comparator(Comparator::NotEqual),
                    source: "!=".to_string(),
                    start_line: *line,
                    start_col: *col - 1,
                    end_line: *line,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 4
        }]);
        let result = lex(input);
        
        assert_eq!(result, expected);
    }
    
    #[test]
    pub fn lexes_string_value() {
        let input = "\"test\"".to_string();
        
        let expected = LinkedList::from([TokenData {
            token: Token::Value("test".to_string()),
            source: "\"test\"".to_string(),
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 6
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
            start_line: 0,
            start_col: 0,
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
            start_line: 0,
            start_col: 0,
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
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start_line: 0,
                start_col: 5,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value("test".to_string()),
                source: "\"test\"".to_string(),
                start_line: 0,
                start_col: 7,
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
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start_line: 0,
                start_col: 4,
                end_line: 0,
                end_col: 5
            },
            TokenData {
                token: Token::Value("test".to_string()),
                source: "\"test\"".to_string(),
                start_line: 0,
                start_col: 5,
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
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start_line: 0,
                start_col: 5,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value("test".to_string()),
                source: "\"test\"".to_string(),
                start_line: 1,
                start_col: 0,
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
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start_line: 0,
                start_col: 5,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value("test".to_string()),
                source: "\"test\"".to_string(),
                start_line: 0,
                start_col: 7,
                end_line: 0,
                end_col: 13
            },
            TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start_line: 0,
                start_col: 14,
                end_line: 0,
                end_col: 15
            },
            TokenData {
                token: Token::Name("test_2".to_string()),
                source: "test_2".to_string(),
                start_line: 0,
                start_col: 16,
                end_line: 0,
                end_col: 22
            },
            TokenData {
                token: Token::Comparator(Comparator::NotEqual),
                source: "!=".to_string(),
                start_line: 0,
                start_col: 24,
                end_line: 0,
                end_col: 26
            },
            TokenData {
                token: Token::Value("test_2".to_string()),
                source: "\"test_2\"".to_string(),
                start_line: 0,
                start_col: 26,
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
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 4
            },
            TokenData {
                token: Token::Comparator(Comparator::Equal),
                source: "=".to_string(),
                start_line: 0,
                start_col: 5,
                end_line: 0,
                end_col: 6
            },
            TokenData {
                token: Token::Value("test".to_string()),
                source: "\"test\"".to_string(),
                start_line: 0,
                start_col: 7,
                end_line: 0,
                end_col: 13
            },
            TokenData {
                token: Token::JoinType(JoinType::Or),
                source: "|".to_string(),
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: 1
            },
            TokenData {
                token: Token::Name("test_2".to_string()),
                source: "test_2".to_string(),
                start_line: 1,
                start_col: 2,
                end_line: 1,
                end_col: 8
            },
            TokenData {
                token: Token::Comparator(Comparator::NotEqual),
                source: "!=".to_string(),
                start_line: 1,
                start_col: 10,
                end_line: 1,
                end_col: 12
            },
            TokenData {
                token: Token::Value("test_2".to_string()),
                source: "\"test_2\"".to_string(),
                start_line: 1,
                start_col: 12,
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