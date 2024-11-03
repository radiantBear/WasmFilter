use std::collections::LinkedList;
use wasm_filter::lexer::*;
use wasm_filter::parser::*;

#[test]
fn parses_with_balanced_joins() {
    let input = "test = \"test\" & test_2 = \"test_2\" | test_3 = \"test_3\" & test_4 = \"test_4\"".to_string();
    
    let expected_parse = Search {
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
    
    let result = lex(input);
    let result = parse(result).unwrap();
    assert_eq!(result, expected_parse);
}
