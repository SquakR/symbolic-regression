use symbolic_regression::expression_tree::{ExpressionTree, Node, ValueNode};

fn main() {
    let expression_tree = ExpressionTree {
        root: Node::Value(ValueNode::Constant(3.0)),
        variables: vec![],
    };
    println!("{}", expression_tree);
}
