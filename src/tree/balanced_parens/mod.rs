use std::marker::PhantomData;
use std::rc::Rc;

use self::parentheses::Parens;
use super::Edge;
use super::LabelVec;
use super::Labels;
use super::NodeId;
use crate::bitwise::ops::ExternalByteSize;
use crate::bitwise::ops::NndOne;
use crate::bitwise::BitString;
use crate::bitwise::Index;
use crate::bitwise::SparseOneNnd;
use crate::tree::traversal::DepthFirstIter;
use crate::tree::traversal::DepthFirstTraverse;

mod parentheses;

pub struct BalancedParensTree<L, N = SparseOneNnd> {
    labels: L,
    parens: Parens<N>,
}
impl<L> BalancedParensTree<LabelVec<L>, SparseOneNnd>
where
    L: Clone,
{
    pub fn new<T>(tree: T) -> Self
    where
        T: DepthFirstTraverse<Label = L>,
    {
        Self::new_builder(tree, LabelVec::new()).build_all()
    }
}
impl<L, N> BalancedParensTree<L, N>
where
    L: Labels,
    N: NndOne + From<BitString>,
{
    pub fn new_builder<T>(tree: T, labels: L) -> Builder<T, L, N>
    where
        T: DepthFirstTraverse<Label = L::Label>,
    {
        Builder::new(tree, labels)
    }
}
impl<L, N> BalancedParensTree<L, N>
where
    L: ExternalByteSize,
    N: ExternalByteSize,
{
    pub fn external_byte_size(&self) -> u64 {
        self.labels.external_byte_size() + self.parens.external_byte_size()
    }
}
impl<L, N> BalancedParensTree<L, N>
where
    L: Labels,
    N: NndOne,
{
    pub fn root(&self) -> Node<L, N, &Self> {
        Node::new(0, 0, self)
    }
    pub fn to_owned_root(self) -> Node<L, N, Rc<Self>> {
        Node::new(0, 0, Rc::new(self))
    }
}

impl<L, N> BalancedParensTree<L, N>
where
    L: Labels,
{
    pub fn len(&self) -> usize {
        self.labels.len()
    }
    pub fn is_empty(&self) -> bool {
        self.labels.len() == 0
    }
}
impl<L, N> BalancedParensTree<L, N> {
    pub fn labels(&self) -> &L {
        &self.labels
    }
}

pub struct Builder<T, L, N = SparseOneNnd> {
    iter: DepthFirstIter<T>,
    labels: L,
    parens: BitString,
    prev_level: usize,
    _nnd: PhantomData<N>,
}
impl<T, L, N> Builder<T, L, N>
where
    T: DepthFirstTraverse,
    L: Labels<Label = T::Label>,
    N: NndOne + From<BitString>,
{
    pub fn new(tree: T, labels: L) -> Self {
        // TODO: Support `with_capacity`
        let mut this = Builder {
            iter: DepthFirstIter::new(tree),
            labels,
            parens: BitString::new(),
            prev_level: 0,
            _nnd: PhantomData,
        };
        this.parens.push(true); // The open parenthesis of the virtual root
        this
    }
    pub fn build_once(&mut self) -> bool {
        if let Some(node) = self.iter.next() {
            let curr_level = node.level + 1;

            for _ in curr_level..self.prev_level + 1 {
                self.parens.push(false);
            }

            self.parens.push(true);
            self.labels.push(node.label);
            self.prev_level = curr_level;
            true
        } else {
            false
        }
    }
    pub fn finish(mut self) -> BalancedParensTree<L, N> {
        for _ in 0..self.prev_level {
            self.parens.push(false);
        }
        self.parens.push(false); // The close parenthesis of the virtual root
        self.labels.shrink_to_fit();
        BalancedParensTree {
            labels: self.labels,
            parens: Parens::new(self.parens), // TODO: incremental
        }
    }
    pub fn build_all(mut self) -> BalancedParensTree<L, N> {
        while self.build_once() {}
        self.finish()
    }
}

pub struct Node<L, N, T> {
    id: NodeId,
    inner_id: NodeId,
    tree: T,
    _n: PhantomData<N>,
    _l: PhantomData<L>,
}
impl<L, N, T> Node<L, N, T>
where
    L: Labels,
    N: NndOne,
    T: ::std::ops::Deref<Target = BalancedParensTree<L, N>> + Clone,
{
    fn new(inner_id: NodeId, id: NodeId, tree: T) -> Self {
        Node {
            id,
            inner_id,
            tree,
            _n: PhantomData,
            _l: PhantomData,
        }
    }
}
impl<L, N, T> Node<L, N, T>
where
    T: Clone,
{
    pub fn tree(&self) -> T {
        self.tree.clone()
    }
}

impl<L, N, T> super::Node<L::Label> for Node<L, N, T>
where
    L: Labels,
    N: NndOne,
    T: ::std::ops::Deref<Target = BalancedParensTree<L, N>> + Clone,
{
    fn id(&self) -> NodeId {
        self.id
    }
    fn first_child(&self) -> Option<Edge<L::Label, Self>> {
        let next = self.inner_id + 1;
        if self.tree.parens.get(next as Index).unwrap() {
            let id = self.id;
            let child = Edge::new(
                self.tree.labels.get(id as usize).unwrap(),
                Self::new(next, id + 1, self.tree.clone()),
            );
            Some(child)
        } else {
            None
        }
    }
    fn next_sibling(&self) -> Option<Edge<L::Label, Self>> {
        let close = self.tree.parens.get_close(self.inner_id as Index).unwrap();
        let next = close + 1;
        if self.tree.parens.get(next).unwrap_or(false) {
            let id = self.id + (close - self.inner_id as Index) as NodeId / 2;
            Some(Edge::new(
                self.tree.labels.get(id as usize).unwrap(),
                Self::new(next as NodeId, id + 1, self.tree.clone()),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::BalancedParensTree;
    use crate::tree::traversal::ByteLines;
    use crate::word::Words;
    use std::io;

    #[test]
    fn it_works() {
        let lines = ByteLines::new(io::Cursor::new(b"aaa\nabc\nd"));
        let tree = BalancedParensTree::new(lines.into_depth_first_traversal());
        assert_eq!(
            Words::new(tree.root())
                .map(|b| String::from_utf8(b).unwrap())
                .collect::<Vec<_>>(),
            ["aaa", "abc", "d"]
        );
    }

    #[test]
    fn it_works2() {
        let input = "aaa111222\nabc3344\nd".to_string();
        let lines = ByteLines::new(io::Cursor::new(input.as_bytes()));
        let tree = BalancedParensTree::new(lines.into_depth_first_traversal());
        assert_eq!(
            Words::new(tree.root())
                .map(|b| String::from_utf8(b).unwrap())
                .collect::<Vec<_>>(),
            ["aaa111222", "abc3344", "d"]
        );
    }
}
