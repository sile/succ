use std::iter;
use std::marker::PhantomData;

use bitwise::Bit;
use bitwise::BitString;
use bitwise::Index;
use bitwise::SparseOneNnd;
use bitwise::ops::NndOne;
use tree::traversal::DepthFirstIter;
use tree::traversal::DepthFirstTraverse;
use super::Edge;
use super::NodeId;
use super::Labels;
use super::LabelVec;
use self::parentheses::Parens;

mod parentheses;

pub struct BalancedParensTree<L, N = SparseOneNnd> {
    labels: L,
    parens: Parens<N>,
}
impl<L> BalancedParensTree<LabelVec<L>, SparseOneNnd>
    where L: Clone
{
    pub fn new<T>(tree: T) -> Result<Self, T::Error>
        where T: DepthFirstTraverse<Label = L>
    {
        Self::new_builder(tree, LabelVec::new()).build_all()
    }
}
impl<L, N> BalancedParensTree<L, N>
    where L: Labels,
          N: NndOne + iter::FromIterator<Bit>
{
    pub fn new_builder<T>(tree: T, labels: L) -> Builder<T, L, N>
        where T: DepthFirstTraverse<Label = L::Label>
    {
        Builder::new(tree, labels)
    }
}
impl<L, N> BalancedParensTree<L, N>
    where L: Labels,
          N: NndOne
{
    pub fn root(&self) -> Node<L, N> {
        Node::new(0, 0, self)
    }
}

impl<L, N> BalancedParensTree<L, N>
    where L: Labels
{
    pub fn len(&self) -> usize {
        self.labels.len()
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
    where T: DepthFirstTraverse,
          L: Labels<Label = T::Label>,
          N: NndOne + iter::FromIterator<Bit>
{
    pub fn new(tree: T, labels: L) -> Self {
        let mut this = Builder {
            iter: DepthFirstIter::new(tree),
            labels: labels,
            parens: BitString::new(),
            prev_level: 0,
            _nnd: PhantomData,
        };
        this.parens.push(Bit::One); // The open parenthesis of the virtual root
        this
    }
    pub fn build_once(&mut self) -> Result<bool, T::Error> {
        if let Some(node) = self.iter.next() {
            let node = try!(node);
            let curr_level = node.level + 1;

            for _ in curr_level..self.prev_level + 1 {
                self.parens.push(Bit::Zero);
            }

            self.parens.push(Bit::One);
            self.labels.push(node.label);
            self.prev_level = curr_level;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn finish(mut self) -> BalancedParensTree<L, N> {
        for _ in 0..self.prev_level {
            self.parens.push(Bit::Zero);
        }
        self.parens.push(Bit::Zero); // The close parenthesis of the virtual root
        self.labels.shrink_to_fit();
        BalancedParensTree {
            labels: self.labels,
            parens: Parens::new(self.parens), // TODO: incremental
        }
    }
    pub fn build_all(mut self) -> Result<BalancedParensTree<L, N>, T::Error> {
        while try!(self.build_once()) {}
        Ok(self.finish())
    }
}

pub struct Node<'a, L: 'a, N: 'a> {
    id: NodeId,
    label_id: usize,
    tree: &'a BalancedParensTree<L, N>,
}
impl<'a, L: 'a, N: 'a> Node<'a, L, N>
    where L: Labels,
          N: NndOne
{
    fn new(id: NodeId, label_id: usize, tree: &'a BalancedParensTree<L, N>) -> Self {
        Node {
            id: id,
            label_id: label_id,
            tree: tree,
        }
    }
}
impl<'a, L, N> super::Node<L::Label> for Node<'a, L, N>
    where L: Labels,
          N: NndOne
{
    fn id(&self) -> NodeId {
        self.id
    }
    fn first_child(&self) -> Option<Edge<L::Label, Self>> {
        let next = self.id + 1;
        if self.tree.parens.get(next as Index).unwrap().is_one() {
            let label_id = self.label_id;
            let child = Edge::new(self.tree.labels.get(label_id).unwrap(),
                                  Self::new(next as NodeId, label_id + 1, &self.tree));
            Some(child)
        } else {
            None
        }
    }
    fn next_sibling(&self) -> Option<Edge<L::Label, Self>> {
        let close = self.tree.parens.get_close(self.id as Index).unwrap();
        let next = close + 1;
        if self.tree.parens.get(next).unwrap_or(Bit::Zero).is_one() {
            assert!(close >= self.id as Index,
                    "assert: {} >= {}",
                    close,
                    self.id);
            let label_id = self.label_id + (close - self.id as Index) as usize / 2;
            Some(Edge::new(self.tree.labels.get(label_id).unwrap(),
                           Self::new(next as NodeId, label_id + 1, &self.tree)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use tree::Node;
    use tree::traversal::ByteLines;
    use word::{Letter, Words};
    use super::BalancedParensTree;

    #[test]
    fn it_works() {
        let lines = ByteLines::new(io::Cursor::new(b"aaa\nabc\nd"));
        let tree = BalancedParensTree::new(lines).unwrap();
        assert_eq!(Words::new(tree.root())
                       .map(|b| String::from_utf8(b).unwrap())
                       .collect::<Vec<_>>(),
                   ["aaa", "abc", "d"]);
    }

    #[test]
    fn it_works2() {
        let input = "aaa111222\nabc3344\nd".to_string();
        let lines = ByteLines::new(io::Cursor::new(input.as_bytes()));
        let tree = BalancedParensTree::new(lines).unwrap();
        assert_eq!(Words::new(tree.root())
                       .map(|b| String::from_utf8(b).unwrap())
                       .collect::<Vec<_>>(),
                   ["aaa111222", "abc3344", "d"]);
    }

    #[test]
    fn it_works3() {
        use std::io::BufRead;
        use std::io::BufReader;

        let input = include_str!("/usr/share/dict/american-english");
        let lines = ByteLines::new(io::Cursor::new(input.as_bytes()));
        let tree = BalancedParensTree::new(lines).unwrap();

        fn label_eq(a: &&u8, b: &Letter<u8>) -> bool {
            **a == b.value
        }
        assert!(tree.root().find_path("Ali".as_bytes().iter(), label_eq).is_some());
        assert!(tree.root().find_path("colitis".as_bytes().iter(), label_eq).is_some());
        assert!(tree.root().find_path("Abner".as_bytes().iter(), label_eq).is_some());
        assert!(tree.root().find_path("Abbas".as_bytes().iter(), label_eq).is_some());
        assert!(tree.root().find_path("Aaliyah".as_bytes().iter(), label_eq).is_some());

        for (i, (w1, w2)) in BufReader::new(io::Cursor::new(input.as_bytes()))
            .lines()
            .zip(Words::new(tree.root()).map(|b| String::from_utf8(b).unwrap()))
            .enumerate() {
            let w1 = w1.unwrap();
            assert_eq!(w1, w2, "[{}] {} == {}", i, w1, w2);
        }
    }
}
