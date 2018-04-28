use std::collections::{HashMap, HashSet};
use std::fmt;

use vocabulary_index::{VocabularyIndex, Words};
use cipher_text::{CipherChar, CipherText, CipherWordId, Condition};
use vocabulary::Char;

const ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";

pub struct Decipher<'r> {
    index: VocabularyIndex,
    cipher_text: &'r CipherText,
}

pub struct Solution {
    solution: Vec<PartialSoultion>,
}

#[derive(Debug)]
pub struct PartialSoultion {
    satisfactory_words: HashMap<CipherWordId, Words>,
}

struct SolutionBuilder {
    current_condition: Vec<PartialSoultion>,
    next_condition: Vec<PartialSoultion>,
}

impl<'r> Decipher<'r> {
    pub fn new(index: VocabularyIndex, cipher_text: &'r CipherText) -> Decipher<'r> {
        Decipher { index, cipher_text }
    }

    pub fn find_solution(&self) -> Solution {
        let mut solutions = SolutionBuilder::new();
        for condition in self.cipher_text.conditions() {
            solutions.next_condition();
            for ch in ALPHABET.chars().map(|ch| Char(ch as u8)) {
                if let Some(partial_soultion) = self.partial_soultion(condition, ch) {
                    solutions.add(partial_soultion);
                }
            }
            debug!("{:?}", &solutions);
        }
        solutions.build()
    }

    fn partial_soultion(&self, condition: &Condition, ch: Char) -> Option<PartialSoultion> {
        let mut satisfactory_words: HashMap<CipherWordId, Words> = HashMap::new();

        for CipherChar {
            position,
            cipher_word_id,
            length,
        } in condition.equal_chars().iter().cloned()
        {
            match self.index.get(length, ch, position) {
                Some(words) => {
                    satisfactory_words
                        .entry(cipher_word_id)
                        .or_insert_with(|| words.clone())
                        .intersect_with(&words);
                }
                None => return None,
            }
        }

        if satisfactory_words.is_empty() || satisfactory_words.values().any(Words::is_empty) {
            None
        } else {
            Some(PartialSoultion { satisfactory_words })
        }
    }
}

impl Solution {
    pub fn partial_soultions(&self) -> &[PartialSoultion] {
        &self.solution
    }
}

impl SolutionBuilder {
    fn new() -> SolutionBuilder {
        SolutionBuilder {
            current_condition: Vec::with_capacity(0),
            next_condition: Vec::with_capacity(1024),
        }
    }

    fn next_condition(&mut self) {
        assert!(self.current_condition.len() == 0 || self.next_condition.len() > 0);
        let next = ::std::mem::replace(&mut self.next_condition, Vec::with_capacity(1024));
        self.current_condition = next;
    }

    fn add(&mut self, partial_soultion: PartialSoultion) {
        if self.current_condition.is_empty() {
            self.next_condition.push(partial_soultion);
            return;
        }
        for current in &self.current_condition {
            if let Some(intersection) = current.intersect(&partial_soultion) {
                self.next_condition.push(intersection);
            }
        }
    }

    fn build(self) -> Solution {
        Solution {
            solution: self.next_condition,
        }
    }
}

impl PartialSoultion {
    pub fn satisfactory_words(&self) -> &HashMap<CipherWordId, Words> {
        &self.satisfactory_words
    }

    fn intersect(&self, other: &PartialSoultion) -> Option<PartialSoultion> {
        let left_words: HashSet<CipherWordId> = self.satisfactory_words.keys().cloned().collect();
        let right_words: HashSet<CipherWordId> = other.satisfactory_words.keys().cloned().collect();
        let all_words: HashSet<CipherWordId> = left_words.union(&right_words).cloned().collect();
        let mut intersection = HashMap::new();
        for word in all_words {
            match (
                self.satisfactory_words.get(&word),
                other.satisfactory_words.get(&word),
            ) {
                (Some(left), None) => {
                    intersection.insert(word, left.clone());
                }
                (None, Some(right)) => {
                    intersection.insert(word, right.clone());
                }
                (Some(left), Some(right)) => {
                    if let Some(word_intersection) = left.intersection(right) {
                        intersection.insert(word, word_intersection);
                    } else {
                        return None;
                    }
                }
                _ => unreachable!(),
            }
        }
        if intersection.is_empty() {
            None
        } else {
            Some(PartialSoultion {
                satisfactory_words: intersection,
            })
        }
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Solution {{ total_entries: {} }}", self.solution.len())
    }
}

impl fmt::Debug for SolutionBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "SolutionBuilder {{ current_condition: {}, next_condition: {} }}",
            self.current_condition.len(),
            self.next_condition.len(),
        )
    }
}
