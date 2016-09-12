pub use self::byte_lines::ByteLines;

mod byte_lines;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisitNode<T> {
    pub label: T,
    pub level: usize,
    pub nth_child: usize,
}
impl<T> VisitNode<T> {
    pub fn new(label: T, level: usize, nth_child: usize) -> Self {
        VisitNode {
            label: label,
            level: level,
            nth_child: nth_child,
        }
    }
}

pub trait DepthFirstTraverse {
    type Label;
    type Error;
    fn next(&mut self) -> Option<Result<VisitNode<Self::Label>, Self::Error>>;
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
    where T: DepthFirstTraverse
{
    type Item = Result<VisitNode<T::Label>, T::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
