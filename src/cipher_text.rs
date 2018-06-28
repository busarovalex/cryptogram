use std::collections::{HashMap, HashSet};
use std::fmt;

use vocabulary::Position;

#[derive(Debug)]
pub struct CipherText {
    text: String,
    word_count: usize,
    conditions: Vec<Condition>,
    lengths: Vec<usize>,
}

#[derive(Clone, Copy)]
pub struct CipherChar {
    pub position: Position,
    pub cipher_word_id: CipherWordId,
    pub length: u8,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CipherWordId(u8);

#[derive(Clone)]
pub struct Condition {
    equal_chars: Vec<CipherChar>,
}

impl CipherText {
    pub fn new(text: String) -> CipherText {
        let mut char_map: HashMap<char, Vec<CipherChar>> = HashMap::new();
        let mut lengths = Vec::new();
        let word_count = text.split_whitespace().enumerate().count();
        for (cipher_word_id, cipher_word) in text.split_whitespace().enumerate() {
            assert!(cipher_word_id < 256);
            assert!(cipher_word.len() < 256);
            let length = cipher_word.len() as u8;
            lengths.push(length as usize);
            for (char_position, ch) in cipher_word.char_indices() {
                char_map
                    .entry(ch)
                    .or_insert_with(Vec::new)
                    .push(CipherChar {
                        position: Position(char_position as u8),
                        cipher_word_id: CipherWordId(cipher_word_id as u8),
                        length,
                    });
            }
        }
        let mut conditions: Vec<Condition> = char_map
            .into_iter()
            .filter(|&(_, ref equal_chars)| equal_chars.len() > 1)
            .map(|(_, equal_chars)| Condition { equal_chars })
            .collect();

        conditions.sort_unstable_by_key(|condition| condition.score());

        conditions.reverse();

        CipherText {
            text,
            conditions,
            word_count,
            lengths,
        }
    }

    pub fn reorder_conditions(&mut self, reorder: &[usize]) {
        assert_eq!(reorder.len(), self.conditions.len());
        let conditions_len = self.conditions.len();
        let reordered_conditions = reorder.iter()
            .map(|index| *index)
            .inspect(|index| assert!(*index <= conditions_len))
            .map(|index| self.conditions[index - 1].clone())
            .collect();
        ::std::mem::replace(&mut self.conditions, reordered_conditions);
    }

    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    pub fn length_of(&self, id: CipherWordId) -> Option<usize> {
        self.lengths.get(id.0 as usize).cloned()
    }
}

impl Condition {
    pub fn equal_chars(&self) -> &[CipherChar] {
        &self.equal_chars
    }

    fn score(&self) -> usize {
        let different_words = self.equal_chars
            .iter()
            .fold(HashSet::new(), |mut set, cipher_char| {
                set.insert(cipher_char.cipher_word_id);
                set
            })
            .len();
        let condition_length = self.equal_chars.len();
        10 * different_words + condition_length
    }
}

impl fmt::Debug for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (index, ch) in self.equal_chars.iter().enumerate() {
            if index < self.equal_chars.len() - 1 {
                write!(f, "{}[{:?}] == ", ch.cipher_word_id.0, ch.position)?;
            } else {
                write!(f, "{}[{:?}]", ch.cipher_word_id.0, ch.position)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for CipherText {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "initial text: \"{}\"\n", &self.text)?;
        for (index, condition) in self.conditions.iter().enumerate() {
            write!(f, "    {}) {}\n", index + 1, &condition)?;
        }
        Ok(())
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (index, ch) in self.equal_chars.iter().enumerate() {
            if index < self.equal_chars.len() - 1 {
                write!(f, "{}[{:?}] == ", ch.cipher_word_id.0, ch.position)?;
            } else {
                write!(f, "{}[{:?}]", ch.cipher_word_id.0, ch.position)?;
            }
        }
        Ok(())
    }
}
