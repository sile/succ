use std::io;

use crate::word::DepthFirstTraversal;

pub struct ByteLines<R> {
    reader: io::Split<R>,
    on_error: Box<dyn Fn(io::Error)>,
}
impl<R> ByteLines<R>
where
    R: io::BufRead,
{
    pub fn new(reader: R) -> Self {
        ByteLines {
            reader: reader.split(b'\n'),
            on_error: Box::new(|e| panic!("Error: {}", e)),
        }
    }
    pub fn set_on_error<F>(&mut self, on_error: F)
    where
        F: Fn(io::Error) + 'static,
    {
        self.on_error = Box::new(on_error);
    }
    pub fn into_depth_first_traversal(self) -> DepthFirstTraversal<u8, Self> {
        DepthFirstTraversal::new(self)
    }
}
impl<R> Iterator for ByteLines<R>
where
    R: io::BufRead,
{
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.next() {
            None => None,
            Some(Ok(line)) => Some(line),
            Some(Err(e)) => {
                (self.on_error)(e);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::VisitNode;
    use super::*;
    use crate::word::Letter;
    use std::io;

    #[test]
    fn it_works() {
        let lines = ByteLines::new(io::Cursor::new(b"aaa\nabc\nd"))
            .into_depth_first_traversal()
            .iter();
        assert_eq!(
            lines.collect::<Vec<_>>(),
            vec![
                VisitNode::new(Letter::new(false, b'a'), 0, 0),
                VisitNode::new(Letter::new(false, b'a'), 1, 0),
                VisitNode::new(Letter::new(true, b'a'), 2, 0),
                VisitNode::new(Letter::new(false, b'b'), 1, 1),
                VisitNode::new(Letter::new(true, b'c'), 2, 0),
                VisitNode::new(Letter::new(true, b'd'), 0, 1)
            ]
        );
    }
}
