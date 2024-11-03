use std::collections::LinkedList;
use crate::lexer::{Comparator, JoinType, Token, TokenData};

#[derive(Debug, Eq, PartialEq)]
pub enum Literal {
    // Number(f64),
    String(String),
    // Bool(bool)
}

#[derive(Debug, Eq, PartialEq)]
pub struct Comparison {
    pub name: String,
    pub comparator: Comparator,
    pub value: Literal
}

#[derive(Debug, Eq, PartialEq)]
pub struct Search {
    pub comparisons: LinkedList<ComparisonOrSearch>,
    pub join_type: JoinType
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComparisonOrSearch {
    Comparison(Comparison),
    Search(Search)
}

pub fn parse(tokens: LinkedList<TokenData>) -> Result<Search, String> {
    let mut tokens = to_postfix(tokens);

    if let Some(comparison_or_search) = _parse(&mut tokens)? {
        match comparison_or_search {
            ComparisonOrSearch::Search(search) => Ok(search),
            comparison@ _ => Ok(Search { comparisons: LinkedList::from([comparison]), join_type: JoinType::And })
        }

    } else {
        Ok(Search { comparisons: LinkedList::new(), join_type: JoinType::And })
    }
}

fn _parse(tokens: &mut LinkedList<TokenData>) -> Result<Option<ComparisonOrSearch>, String> {
    if tokens.is_empty() {
        return Ok(None);
    }

    match tokens.pop_back().unwrap().token {
        Token::JoinType(join_type) => {
            let right_tree = _parse(tokens)?;
            let left_tree = _parse(tokens)?;

            
            let mut search = Search{
                join_type,
                comparisons: LinkedList::new()
            };
            merge_subtree(&mut search, left_tree);
            merge_subtree(&mut search, right_tree);

            Ok(Some(ComparisonOrSearch::Search(search)))
        }

        Token::Value(value) => {
            let Token::Comparator(comparator) = tokens.pop_back().unwrap().token else { panic!("Expected comparator") };
            let Token::Name(name) = tokens.pop_back().unwrap().token else { panic!("Expected name") };

            Ok(Some(ComparisonOrSearch::Comparison(Comparison {
                name,
                comparator,
                value: Literal::String(value)
            })))
        }

        token @ _ => Err(format!("Unexpected token {:?}", token).to_string())
    }
}

fn merge_subtree(search: &mut Search, subtree: Option<ComparisonOrSearch>) {
    if let Some(mut subtree) = subtree {
        if let ComparisonOrSearch::Search(ref mut subsearch) = subtree {
            if subsearch.join_type == search.join_type {
                search.comparisons.append(&mut subsearch.comparisons);
            }
            else {
                search.comparisons.push_back(subtree);
            }
        }
        else {
            search.comparisons.push_back(subtree);
        }
    }
}


fn to_postfix(mut tokens: LinkedList<TokenData>) -> LinkedList<TokenData> {
    let mut last_was_join = false;
    let mut postfix = LinkedList::new();
    let mut operator_stack = LinkedList::new();

    while !tokens.is_empty() {
        let token = tokens.pop_front().unwrap();

        match &token.token {
            Token::OpenParen => {
                if !last_was_join {
                    panic!("Expected operator but found open parentheses");
                }
                operator_stack.push_front(token);
            },
            Token::CloseParen => {
                if last_was_join {
                    panic!("Unexpected close parentheses after operator");
                }

                loop {
                    let Some(operator) = operator_stack.pop_front() else {
                        panic!("Close parentheses was found without a preceding open parentheses");
                    };
                    match &operator.token {
                        Token::JoinType(_) => { postfix.push_back(operator) },
                        Token::OpenParen => break,
                        _ => panic!("Invalid token {:?} found in operator stack", operator) 
                    }
                }
            },
            Token::JoinType(join_type) => {
                last_was_join = true;
                while let Some(operator) = operator_stack.front() {
                    match &operator.token {
                        Token::JoinType(operator) => {
                            if operator < join_type {
                                break;
                            }
                            // Now know that the operator at the stack's top is higher precedence than the new operator, meaning we want to move
                            // it to `postfix`, so we can now safely remove it from `operator_stack` instead of just using `.front()`. Also
                            // needed to actually let us perform a move into the `postfix` LinkedList.
                            let operator = operator_stack.pop_front().unwrap();
                            postfix.push_back(operator);
                        }
                        Token::OpenParen => {
                            // Everything inside parentheses should have higher precedence than the stuff outside
                            break;
                        },
                        _ => panic!("Invalid token {:?} found in operator stack", operator)
                    }
                }
                operator_stack.push_front(token);
            },
            _ => {
                last_was_join = false;
                postfix.push_back(token);
            }
        }
    }

    while !operator_stack.is_empty() {
        let next_op = operator_stack.pop_front().unwrap();
        match &next_op.token {
            Token::JoinType(_) => postfix.push_back(next_op),
            Token::OpenParen => panic!("Unclosed parentheses!"),
            _ => panic!("Invalid token {:?} found in operator stack", next_op)
        }
    }

    postfix
}


#[cfg(test)]
mod parser_tests {
    use super::*;
    
    // Note: to_postfix_tests module ensures that order of operations & parentheses are handled correctly. No need to include tests for those features here.

    #[test]
    fn parses_single_comparison() {
        let input = LinkedList::from([ 
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 }
        ]);

        let expected = LinkedList::from([ ComparisonOrSearch::Comparison(Comparison{
            name: "test".to_string(), comparator: Comparator::Equal, value: Literal::String("test".to_string())
        })]);
        let result = parse(input);

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.comparisons, expected);
    }

    #[test]
    fn parses_single_join() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },
        ]);

        let expected = Search {
            comparisons: LinkedList::from([
                ComparisonOrSearch::Comparison(Comparison{ name: "test".to_string(), comparator: Comparator::Equal, value: Literal::String("test".to_string()) }),
                ComparisonOrSearch::Comparison(Comparison{ name: "test_2".to_string(), comparator: Comparator::Equal, value: Literal::String("test_2".to_string()) })
            ]),
            join_type: JoinType::Or
        };
        let result = parse(input);

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result, expected);
    }
    
    #[test]
    fn combines_repeated_joins() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },
            
            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::Name("test_4".to_string()), source: "test_4".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_4".to_string()), source: "\"test_4\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = Search {
            comparisons: LinkedList::from([
                ComparisonOrSearch::Comparison(Comparison{ name: "test".to_string(), comparator: Comparator::Equal, value: Literal::String("test".to_string()) }),
                ComparisonOrSearch::Comparison(Comparison{ name: "test_2".to_string(), comparator: Comparator::Equal, value: Literal::String("test_2".to_string()) }),
                ComparisonOrSearch::Comparison(Comparison{ name: "test_3".to_string(), comparator: Comparator::Equal, value: Literal::String("test_3".to_string()) }),
                ComparisonOrSearch::Comparison(Comparison{ name: "test_4".to_string(), comparator: Comparator::Equal, value: Literal::String("test_4".to_string()) })
            ]),
            join_type: JoinType::And
        };
        let result = parse(input);

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn parses_balanced_nested_join() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_4".to_string()), source: "test_4".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_4".to_string()), source: "\"test_4\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = Search {
            comparisons: LinkedList::from([
                ComparisonOrSearch::Search(Search {
                    comparisons: LinkedList::from([
                        ComparisonOrSearch::Comparison(Comparison{ name: "test".to_string(), comparator: Comparator::Equal, value: Literal::String("test".to_string()) }),
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_2".to_string(), comparator: Comparator::Equal, value: Literal::String("test_2".to_string()) })
                    ]),
                    join_type: JoinType::And
                }),
                ComparisonOrSearch::Search(Search {
                    comparisons: LinkedList::from([
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_3".to_string(), comparator: Comparator::Equal, value: Literal::String("test_3".to_string()) }),
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_4".to_string(), comparator: Comparator::Equal, value: Literal::String("test_4".to_string()) })
                    ]),
                    join_type: JoinType::And
                })
            ]),
            join_type: JoinType::Or
        };
        let result = parse(input);

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result, expected);
    }
    
    #[test]
    fn parses_imbalanced_nested_join() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_4".to_string()), source: "test_4".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_4".to_string()), source: "\"test_4\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = Search {
            comparisons: LinkedList::from([
                ComparisonOrSearch::Comparison(Comparison{ name: "test".to_string(), comparator: Comparator::Equal, value: Literal::String("test".to_string()) }),
                ComparisonOrSearch::Search(Search {
                    comparisons: LinkedList::from([
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_2".to_string(), comparator: Comparator::Equal, value: Literal::String("test_2".to_string()) }),
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_3".to_string(), comparator: Comparator::Equal, value: Literal::String("test_3".to_string()) }),
                        ComparisonOrSearch::Comparison(Comparison{ name: "test_4".to_string(), comparator: Comparator::Equal, value: Literal::String("test_4".to_string()) })
                    ]),
                    join_type: JoinType::And
                })
            ]),
            join_type: JoinType::Or
        };
        let result = parse(input);

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod to_postfix_tests {
    use super::*;

    #[test]
    fn leaves_comparisons_alone() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn moves_single_join_type_to_end() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn gives_and_precedence_over_or() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn gives_and_precedence_over_or_2() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn gives_xor_precedence_over_and() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },
            
            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn gives_xor_precedence_over_and_2() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 22 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 23, end_line: 0, end_col: 24 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 25, end_line: 0, end_col: 33 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 34, end_line: 0, end_col: 35 },
            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn parentheses_override_precedence_and_over_or() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },
            
            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },
            
            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },
            
            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
    
            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn parentheses_override_precedence_xor_over_and() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn correctly_transforms_complex_expressions() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },
            
            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },
            
            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_4".to_string()), source: "test_4".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThan), source: ">".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_4".to_string()), source: "\"test_4\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_5".to_string()), source: "test_5".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThanOrEqual), source: ">=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_5".to_string()), source: "\"test_5\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_6".to_string()), source: "test_6".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThanOrEqual), source: ">=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_6".to_string()), source: "\"test_6\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
        ]);

        let expected = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },


            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_4".to_string()), source: "test_4".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThan), source: ">".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_4".to_string()), source: "\"test_4\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_5".to_string()), source: "test_5".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThanOrEqual), source: ">=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_5".to_string()), source: "\"test_5\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },


            TokenData{ token: Token::JoinType(JoinType::Xor), source: "^".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_6".to_string()), source: "test_6".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::GreaterThanOrEqual), source: ">=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_6".to_string()), source: "\"test_6\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);
        let result = to_postfix(input);

        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "without a preceding open")]
    fn panics_if_given_close_paren_without_open() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        to_postfix(input);
    }

    #[test]
    #[should_panic(expected = "Unclosed")]
    fn panics_if_given_open_paren_without_close() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
        ]);

        to_postfix(input);
    }

    #[test]
    #[should_panic(expected = "Unclosed")]
    fn panics_on_bad_nested_parens() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 18, end_line: 0, end_col: 19 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            
            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
        ]);

        to_postfix(input);
    }

    #[test]
    #[should_panic(expected = "Expected operator")]
    fn panics_on_out_of_order_open_parentheses() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },
            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
        ]);

        to_postfix(input);
    }

    #[test]
    #[should_panic(expected = "Unexpected close")]
    fn panics_on_out_of_order_close_parentheses() {
        let input = LinkedList::from([
            TokenData{ token: Token::Name("test".to_string()), source: "test".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 4 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 5, end_line: 0, end_col: 6 },
            TokenData{ token: Token::Value("test".to_string()), source: "\"test\"".to_string(), start_line: 0, start_col: 7, end_line: 0, end_col: 13 },

            TokenData{ token: Token::JoinType(JoinType::And), source: "&".to_string(), start_line: 0, start_col: 14, end_line: 0, end_col: 15 },
            TokenData{ token: Token::OpenParen, source: "(".to_string(), start_line: 0, start_col: 16, end_line: 0, end_col: 17 },

            TokenData{ token: Token::Name("test_2".to_string()), source: "test_2".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_2".to_string()), source: "\"test_2\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::JoinType(JoinType::Or), source: "|".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::CloseParen, source: ")".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },

            TokenData{ token: Token::Name("test_3".to_string()), source: "test_3".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Comparator(Comparator::Equal), source: "=".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
            TokenData{ token: Token::Value("test_3".to_string()), source: "\"test_3\"".to_string(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 },
        ]);

        to_postfix(input);
    }
}