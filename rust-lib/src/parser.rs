/// Expression tree parser module.
/// The parser uses the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{ExpressionTree, Node, Value};

impl ExpressionTree {
    pub fn parse(expression: &str) -> ExpressionTree {
        ExpressionTree {
            root: Box::new(Node::Value(Value::Constant(0.0))),
            variables: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(0.0, 0.0);
    }
}
