use super::DepthFirstIter;
use super::DepthFirstTraverse;
use super::VisitNode;
use crate::tree::Edge;
use crate::tree::Node;

type Level = usize;
type NthChild = usize;

pub struct TreeTraversal<L, N> {
    stack: Vec<(Edge<L, N>, Level, NthChild)>,
}
impl<L, N> TreeTraversal<L, N>
where
    N: Node<L>,
{
    pub fn new(root: N) -> Self {
        TreeTraversal {
            stack: root.first_child().into_iter().map(|e| (e, 0, 0)).collect(),
        }
    }
    pub fn into_depth_first_iter(self) -> DepthFirstIter<Self> {
        DepthFirstIter::new(self)
    }
}
impl<L, N> DepthFirstTraverse for TreeTraversal<L, N>
where
    N: Node<L>,
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

pub struct PatriciaTreeTraversal<L, N> {
    stack: Vec<(Edge<Vec<L>, N>, Level, NthChild)>,
}
impl<L, N> PatriciaTreeTraversal<L, N>
where
    N: Node<L>,
{
    pub fn new(root: N) -> Self {
        PatriciaTreeTraversal {
            stack: root
                .first_child()
                .into_iter()
                .map(|e| (Edge::new(vec![e.label], e.node), 0, 0))
                .collect(),
        }
    }
    pub fn into_depth_first_iter(self) -> DepthFirstIter<Self> {
        DepthFirstIter::new(self)
    }
}
impl<L, N> DepthFirstTraverse for PatriciaTreeTraversal<L, N>
where
    N: Node<L>,
{
    type Label = Vec<L>;
    fn next(&mut self) -> Option<VisitNode<Self::Label>> {
        if let Some((mut edge, level, nth_child)) = self.stack.pop() {
            let mut has_sibling = false;
            if let Some(s) = edge.node.next_sibling() {
                has_sibling = true;
                self.stack
                    .push((Edge::new(vec![s.label], s.node), level, nth_child + 1));
            }
            if let Some(c) = edge.node.first_child() {
                if has_sibling {
                    let c = Edge::new(vec![c.label], c.node);
                    self.stack.push((c, level + 1, 0));
                    let visit = VisitNode::new(edge.label, level, nth_child);
                    Some(visit)
                } else {
                    edge.label.push(c.label);
                    let c = Edge::new(edge.label, c.node);
                    self.stack.push((c, level, 0));
                    self.next()
                }
            } else {
                let visit = VisitNode::new(edge.label, level, nth_child);
                Some(visit)
            }
        } else {
            None
        }
    }
}
