use std::marker::PhantomData;
use std::mem;

use crate::bitwise::ops::ExternalByteSize;

pub mod balanced_parens;
pub mod traversal;

pub type NodeId = u32;

pub trait Node<L>: Sized {
    fn id(&self) -> NodeId;
    fn first_child(&self) -> Option<Edge<L, Self>>;
    fn next_sibling(&self) -> Option<Edge<L, Self>>;
    fn children(&self) -> Children<Self, L> {
        Children::new(self)
    }
    fn find_path<P, M, F>(&self, path: P, f: F) -> Option<Self>
    where
        P: Iterator<Item = M>,
        F: Fn(&M, &L) -> bool,
    {
        let mut children = self.children();
        let mut last_child = None;
        for label in path {
            match children.find(|e| f(&label, &e.label)) {
                None => return None,
                Some(c) => {
                    children = c.node.children();
                    last_child = Some(c.node);
                }
            }
        }
        last_child
    }
    fn into_tree_traversal(self) -> traversal::TreeTraversal<L, Self> {
        traversal::TreeTraversal::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Edge<L, N> {
    pub label: L,
    pub node: N,
}
impl<L, N> Edge<L, N> {
    pub fn new(label: L, node: N) -> Self {
        Edge { label, node }
    }
}

pub trait Labels {
    type Label;
    fn push(&mut self, label: Self::Label);
    fn get(&self, index: usize) -> Option<Self::Label>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn shrink_to_fit(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct LabelVec<T>(Vec<T>);
impl<T> LabelVec<T> {
    pub fn new() -> Self {
        LabelVec(Vec::new())
    }
}
impl<T> Default for LabelVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> ExternalByteSize for LabelVec<T>
where
    T: Sized,
{
    fn external_byte_size(&self) -> u64 {
        mem::size_of_val(&self.0.len()) as u64 + mem::size_of::<T>() as u64 * self.0.len() as u64
    }
}
impl<T> Labels for LabelVec<T>
where
    T: Clone,
{
    type Label = T;
    fn push(&mut self, label: Self::Label) {
        self.0.push(label);
    }
    fn get(&self, index: usize) -> Option<Self::Label> {
        self.0.get(index).cloned()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
}

pub struct Children<N, L>
where
    N: Node<L>,
{
    child: Option<Edge<L, N>>,
    _l: PhantomData<L>,
}
impl<N, L> Children<N, L>
where
    N: Node<L>,
{
    fn new(node: &N) -> Self {
        Children {
            child: node.first_child(),
            _l: PhantomData,
        }
    }
}
impl<N, L> Iterator for Children<N, L>
where
    N: Node<L>,
{
    type Item = Edge<L, N>;
    fn next(&mut self) -> Option<Self::Item> {
        self.child.take().inspect(|e| {
            self.child = e.node.next_sibling();
        })
    }
}
