pub use self::byte_lines::ByteLines;
pub use self::tree_traversal::PatriciaTreeTraversal;
pub use self::tree_traversal::TreeTraversal;

mod byte_lines;
mod tree_traversal;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisitNode<T> {
    pub label: T,
    pub level: usize,
    pub nth_child: usize,
}
impl<T> VisitNode<T> {
    pub fn new(label: T, level: usize, nth_child: usize) -> Self {
        VisitNode {
            label,
            level,
            nth_child,
        }
    }
}

pub trait DepthFirstTraverse {
    type Label;
    fn next(&mut self) -> Option<VisitNode<Self::Label>>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub struct DepthFirstIter<T>(T);
impl<T> DepthFirstIter<T> {
    pub fn new(traversal: T) -> Self {
        DepthFirstIter(traversal)
    }
}
impl<T> Iterator for DepthFirstIter<T>
where
    T: DepthFirstTraverse,
{
    type Item = VisitNode<T::Label>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
