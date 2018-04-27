use std::collections::{HashMap, HashSet};
use std::fmt;

use vocabulary::{Char, Position, Vocabulary, WordId};

#[derive(Debug)]
pub struct VocabularyIndex {
    indexes: HashMap<u8, Index>,
}

#[derive(Debug, Clone)]
pub struct Words {
    words: HashSet<WordId>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Key {
    char: Char,
    position: Position,
}

struct Index {
    word_length: u8,
    map: HashMap<Key, HashSet<WordId>>,
}

impl Words {
    pub fn intersect_with(&mut self, other: &Words) {
        self.words = self.words.intersection(&other.words).cloned().collect();
    }

    pub fn intersection(&self, other: &Words) -> Option<Words> {
        let intersection: HashSet<_> = self.words.intersection(&other.words).cloned().collect();
        if intersection.is_empty() {
            None
        } else {
            Some(Words{words: intersection})
        }
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn ids(&self) -> &HashSet<WordId> {
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
            .or_insert_with(HashSet::new)
            .insert(word);
    }

    fn get<T: Into<Key>>(&self, key: T) -> Option<Words> {
        let key: Key = key.into();
        assert!(key.position.0 <= self.word_length - 1);
        self.map.get(&key).map(|words| Words { words: words.clone() })
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
