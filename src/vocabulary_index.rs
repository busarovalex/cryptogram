use std::collections::{HashMap};
use std::fmt;

use vocabulary::{Char, Position, Vocabulary, WordId};

#[derive(Debug)]
pub struct VocabularyIndex {
    indexes: HashMap<u8, Index>,
}

#[derive(Debug, Clone)]
pub struct Words {
    words: Vec<WordId>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Key {
    char: Char,
    position: Position,
}

struct Index {
    word_length: u8,
    map: HashMap<Key, Words>,
}

impl Words {
    pub fn intersect_with(&mut self, other: &Words) {
        self.words = self.intersection(&other)
            .map(|w| w.words)
            .unwrap_or_else(|| Vec::with_capacity(0));
    }

    pub fn intersection(&self, other: &Words) -> Option<Words> {
        let mut result = Vec::with_capacity(::std::cmp::min(self.words.len(), other.words.len()));
        let mut left_iter = self.words.iter();
        let mut right_iter = other.words.iter();

        let mut next_left = left_iter.next();
        let mut next_right = right_iter.next();

        while let (Some(left), Some(right)) = (next_left, next_right) {
            if left > right {
                next_right = right_iter.next();
            } else if left < right {
                next_left = left_iter.next();
            } else {
                result.push(*left);
                next_left = left_iter.next();
                next_right = right_iter.next();
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(Words { words: result })
        }
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn ids(&self) -> &[WordId] {
        &self.words
    }
}

impl VocabularyIndex {
    pub fn new(vocabulary: &Vocabulary) -> VocabularyIndex {
        let mut indexes = HashMap::new();
        for (words_len, words) in vocabulary.by_length().iter().enumerate() {
            assert!(words_len <= 255);
            let words_len = words_len as u8;
            let mut current_word_len_index = Index::new(words_len, words.len());
            for &(word_id, word) in words {
                for (index, ch) in word.char_indices() {
                    current_word_len_index.insert(Key::new(ch, index as u8), word_id);
                }
            }
            indexes.insert(words_len, current_word_len_index);
        }

        for index in indexes.values_mut() {
            for words in index.map.values_mut() {
                words.words.sort_unstable();
            }
        }

        VocabularyIndex { indexes }
    }

    pub fn get(&self, word_length: u8, ch: Char, position: Position) -> Option<Words> {
        self.indexes
            .get(&word_length)
            .and_then(|index| index.get(Key::new(ch, position)))
    }
}

impl Key {
    fn new<T: Into<Char>, S: Into<Position>>(ch: T, position: S) -> Key {
        Key {
            char: ch.into(),
            position: position.into(),
        }
    }
}

impl Index {
    fn new(word_length: u8, capacity: usize) -> Index {
        Index {
            word_length,
            map: HashMap::with_capacity(capacity),
        }
    }

    fn insert<T: Into<Key>>(&mut self, key: T, word: WordId) {
        let key: Key = key.into();
        assert!(key.position.0 <= self.word_length - 1);
        self.map
            .entry(key)
            .or_insert_with(|| Words { words: Vec::new() })
            .words
            .push(word);
    }

    fn get<T: Into<Key>>(&self, key: T) -> Option<Words> {
        let key: Key = key.into();
        assert!(key.position.0 <= self.word_length - 1);
        self.map.get(&key).cloned()
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let total_entries: usize = self.map.values().map(|s| s.len()).sum();
        write!(
            f,
            "Index {{ word_length: {}, total_entries: {} }}",
            self.word_length, total_entries
        )
    }
}
