use std::io;

use word::Letter;
use super::Node;
use super::DepthFirstIter;
use super::DepthFirstTraverse;

pub struct ByteLines<R> {
    buf: Vec<u8>,
    path: Vec<(Letter<u8>, usize)>,
    reader: io::Split<R>,
}
impl<R> ByteLines<R>
    where R: io::BufRead
{
    pub fn new(reader: R) -> Self {
        ByteLines {
            buf: Vec::new(),
            path: vec![(Letter::new(false, 0), 0)],
            reader: reader.split(b'\n'),
        }
    }
    pub fn into_depth_first_iter(self) -> DepthFirstIter<Self> {
        DepthFirstIter::new(self)
    }
}
impl<R> DepthFirstTraverse for ByteLines<R>
    where R: io::BufRead
{
    type Label = Letter<u8>;
    type Error = io::Error;
    fn next(&mut self) -> Option<Result<Node<Self::Label>, Self::Error>> {
        loop {
            if self.path.len() <= self.buf.len() {
                let level = self.path.len() - 1;
                let is_terminal = self.path.len() == self.buf.len();
                let label = Letter::new(is_terminal, self.buf[level]);
                let nth_child = self.path[level].1;
                self.path.push((label.clone(), 0));
                let node = super::Node::new(label, level, nth_child);
                return Some(Ok(node));
            } else {
                match self.reader.next() {
                    Some(Ok(v)) => {
                        self.buf = v;
                        if let Some(tail) = self.path
                            .iter()
                            .skip(1)
                            .zip(self.buf.iter())
                            .position(|(&(ref l, _), &b)| l.value != b) {
                            self.path.truncate(tail + 1);
                            self.path[tail].1 += 1;
                        }
                    }
                    None => return None,
                    Some(Err(e)) => return Some(Err(e)),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use word::Letter;
    use super::super::Node;
    use super::*;

    #[test]
    fn it_works() {
        let lines = ByteLines::new(io::Cursor::new(b"aaa\nabc\nd")).into_depth_first_iter();
        assert_eq!(lines.map(|r| r.unwrap()).collect::<Vec<_>>(),
                   vec![Node::new(Letter::new(false, b'a'), 0, 0),
                        Node::new(Letter::new(false, b'a'), 1, 0),
                        Node::new(Letter::new(true, b'a'), 2, 0),
                        Node::new(Letter::new(false, b'b'), 1, 1),
                        Node::new(Letter::new(true, b'c'), 2, 0),
                        Node::new(Letter::new(true, b'd'), 0, 1)]);
    }
}
