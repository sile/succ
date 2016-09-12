use tree::Node;
use tree::Edge;
use super::VisitNode;
use super::DepthFirstTraverse;

type Level = usize;
type NthChild = usize;

pub struct TreeTraversal<L, N> {
    stack: Vec<(Edge<L, N>, Level, NthChild)>,
}
impl<L, N> TreeTraversal<L, N>
    where N: Node<L>
{
    pub fn new(root: N) -> Self {
        TreeTraversal { stack: root.first_child().into_iter().map(|e| (e, 0, 0)).collect() }
    }
}
impl<L, N> DepthFirstTraverse for TreeTraversal<L, N>
    where N: Node<L>
{
    type Label = L;
    fn next(&mut self) -> Option<VisitNode<Self::Label>> {
        if let Some((edge, level, nth_child)) = self.stack.pop() {
            let visit = VisitNode::new(edge.label, level, nth_child);
            if let Some(sibling) = edge.node.next_sibling() {
                self.stack.push((sibling, level, nth_child + 1));
            }
            if let Some(child) = edge.node.first_child() {
                self.stack.push((child, level + 1, 0));
            }
            Some(visit)
        } else {
            None
        }
    }
}
