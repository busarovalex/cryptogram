use std::fmt;

pub struct Vocabulary<'r> {
    all: &'r [&'r str],
    by_length: Vec<Vec<(WordId, &'r str)>>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Char(pub u8);

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Position(pub u8);

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct WordId(usize);

impl<'r> Vocabulary<'r> {
    pub fn new(words: &'r [&'r str]) -> Vocabulary<'r> {
        let mut by_length = Vec::new();
        for (word_index, word) in words.iter().enumerate() {
            if by_length.len() < word.len() + 1 {
                for _ in 0..word.len() - by_length.len() + 1 {
                    by_length.push(Vec::new());
                }
            }
            by_length[word.len()].push((WordId(word_index), *word));
        }
        Vocabulary {
            all: words,
            by_length,
        }
    }

    pub fn get(&self, word_id: WordId) -> Option<&str> {
        self.all.get(word_id.0).map(|r| *r)
    }

    pub fn by_length(&self) -> &[Vec<(WordId, &str)>] {
        &self.by_length
    }
}

impl fmt::Debug for WordId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl<'r> fmt::Debug for Vocabulary<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut by_length = String::with_capacity(128);
        for (length, words) in self.by_length.iter().enumerate() {
            by_length.push_str(&format!("{}-{},", length, words.len()));
        }
        write!(
            f,
            "Vocabulary {{ all: {}, by_length: [{}] }}",
            self.all.len(),
            by_length
        )
    }
}

impl From<u8> for Char {
    fn from(val: u8) -> Char {
        Char(val)
    }
}

impl From<char> for Char {
    fn from(val: char) -> Char {
        assert!(val <= 'z');
        assert!(val >= 'a');
        Char(val as u8)
    }
}

impl From<u8> for Position {
    fn from(val: u8) -> Position {
        Position(val)
    }
}
