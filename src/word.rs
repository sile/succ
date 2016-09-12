use tree::Node;
use tree::Edge;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Letter<T> {
    pub end_of_word: bool,
    pub value: T,
}
impl<T> Letter<T> {
    pub fn new(end_of_word: bool, value: T) -> Self {
        Letter {
            end_of_word: end_of_word,
            value: value,
        }
    }
}

#[derive(Debug)]
pub struct Words<T, N> {
    buf: Vec<T>,
    stack: Vec<Vec<Edge<Letter<T>, N>>>,
}
impl<T, N> Words<T, N>
    where N: Node<Letter<T>>
{
    pub fn new(root: N) -> Self {
        let mut words = Words {
            buf: Vec::new(),
            stack: Vec::new(),
        };
        let mut children = root.children().collect::<Vec<_>>();
        if !children.is_empty() {
            children.reverse();
            words.stack.push(children);
        }
        words
    }
}
impl<T, N> Iterator for Words<T, N>
    where N: Node<Letter<T>>,
          T: Clone
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut children) = self.stack.pop() {
            if let Some(e) = children.pop() {
                self.stack.push(children);
                self.buf.push(e.label.value);

                let mut grand_children = e.node.children().collect::<Vec<_>>();
                grand_children.reverse();
                self.stack.push(grand_children);

                if e.label.end_of_word {
                    let word = self.buf.clone();
                    return Some(word);
                }
            } else {
                self.buf.pop();
            }
        }
        debug_assert!(self.buf.is_empty());
        None
    }
}
