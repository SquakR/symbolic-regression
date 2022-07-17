mod expression_tree;

#[cfg(test)]
mod expression_tree_tests {
    use super::*;
    use expression_tree::Node;

    #[test]
    fn it_works() {
        let node = Node {};
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
