use std::io::{Error, ErrorKind};

pub struct ByteStream<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> ByteStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        return ByteStream { data, index: 0 };
    }

    // It is your responsability to ensure it's not empty before peeking.
    pub fn peek(&self) -> Option<u8> {
        if self.index >= self.data.len() {
            return None;
        }
        Some(self.data[self.index])
    }

    pub fn len(&self) -> usize {
        let len = self.data.len();
        if self.index >= len {
            return 0;
        }
        len - self.index
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn consume(&mut self, amount: usize) -> Result<&'a [u8], Error> {
        if self.len() < amount {
            return Err(Error::new(ErrorKind::UnexpectedEof, "consume"));
        }

        let result = &self.data[self.index..(self.index + amount)];
        self.index += amount;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let data = &[1, 2, 3];
        let stream = ByteStream::new(data);
        assert_eq!(stream.index, 0);
        assert_eq!(stream.data, data);
    }

    #[test]
    fn test_peek() {
        let data = &[1, 2, 3];
        let mut stream = ByteStream::new(data);
        assert_eq!(stream.peek().unwrap(), 1);
        stream.consume(2).unwrap();
        assert_eq!(stream.peek().unwrap(), 3);
    }

    #[test]
    fn test_peek_empty() {
        let data: &[u8] = &[];
        let stream = ByteStream::new(data);
        assert!(stream.peek().is_none());
    }

    #[test]
    fn test_len() {
        let data = &[1, 2, 3];
        let mut stream = ByteStream::new(data);
        assert_eq!(stream.len(), 3);
        stream.consume(2).unwrap();
        assert_eq!(stream.len(), 1);
    }

    #[test]
    fn test_consume() {
        let data = &[1, 2, 3, 4, 5, 6];
        let mut stream = ByteStream::new(data);
        assert_eq!(stream.consume(2).unwrap(), &[1, 2]);
        assert_eq!(stream.peek().unwrap(), 3);
        assert_eq!(stream.consume(2).unwrap(), &[3, 4]);
        assert_eq!(stream.consume(2).unwrap(), &[5, 6]);
    }

    #[test]
    fn test_consume_too_much() {
        let data = &[1, 2, 3];
        let mut stream = ByteStream::new(data);
        assert!(stream.consume(4).is_err());
    }

    #[test]
    fn test_is_empty() {
        let data = &[1, 2];
        let mut stream = ByteStream::new(data);
        assert!(!stream.is_empty());
        stream.consume(2).unwrap();
        assert!(stream.is_empty());
    }
}
