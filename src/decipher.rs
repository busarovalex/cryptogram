use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::Iterator;

use vocabulary_index::{VocabularyIndex, Words};
use cipher_text::{CipherChar, CipherText, CipherWordId, Condition};
use vocabulary::{AlphabetIter, Char};

pub struct Decipher<'r> {
    index: VocabularyIndex,
    cipher_text: &'r CipherText,
}

pub struct Solution {
    solution: Vec<PartialSolution>,
}

#[derive(Debug)]
pub struct PartialSolution {
    satisfactory_words: HashMap<CipherWordId, Words>,
}

struct BacktrackingSearch<'r> {
    rules: &'r [Condition],
    solutions: Vec<(AlphabetIter, PartialSolution)>,
    current: AlphabetIter,
    full_solutions: Vec<PartialSolution>,
    index: &'r VocabularyIndex
}

impl<'r> Decipher<'r> {
    pub fn new(index: VocabularyIndex, cipher_text: &'r CipherText) -> Decipher<'r> {
        Decipher { index, cipher_text }
    }

    pub fn find_solution(&self) -> Solution {
        let mut search = BacktrackingSearch {
            rules:   self.cipher_text.conditions(),
            solutions: Vec::new(),
            current: AlphabetIter::new(),
            full_solutions: Vec::new(),
            index: &self.index
        };

        loop {
            if let Some(next_char) = search.current.next() {
                if search.solutions.len() == 0 {
                    debug!("{}% complete", ((next_char.0 - 'a' as u8) as f32 / 26.) * 100.);
                }
                if let Some(solution) = search.partial_solution_intersected_with_top_solution(next_char) {
                    if search.solutions.len() == search.rules.len() - 1 {
                        search.full_solutions.push(solution);
                        let solution_count = search.full_solutions.len();
                        if solution_count % 10 == 0 {
                            debug!("found solution №{}", search.full_solutions.len());
                        }
                        if search.full_solutions.len() > 10_000 {
                            println!("Too many solutions!");
                            ::std::process::exit(1);
                        }
                    } else {
                        let current_char_iter = ::std::mem::replace(&mut search.current, AlphabetIter::new());
                        search.solutions.push((current_char_iter, solution));
                    }
                }
            } else if search.solutions.is_empty() {
                break;
            } else if let Some((last_char_iter, _)) = search.solutions.pop() {
                search.current = last_char_iter;
            }
        }

        Solution {
            solution: search.full_solutions
        }
    }
}

impl<'r> BacktrackingSearch<'r> {
    fn partial_solution_intersected_with_top_solution(&self, ch: Char) -> Option<PartialSolution> {
        let found = self.partial_solution(self.current_rule(), ch)?;
        if let Some(&(_, ref last)) = self.solutions.last() {
            return last.intersect(&found);
        }
        Some(found)
    }

    fn current_rule(&self) -> &Condition {
        &self.rules[self.solutions.len()]
    }

    fn partial_solution(&self, condition: &Condition, ch: Char) -> Option<PartialSolution> {
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
            Some(PartialSolution { satisfactory_words })
        }
    }
}

impl Solution {
    pub fn partial_solutions(&self) -> &[PartialSolution] {
        &self.solution
    }
}

impl PartialSolution {
    pub fn satisfactory_words(&self) -> &HashMap<CipherWordId, Words> {
        &self.satisfactory_words
    }

    fn intersect(&self, other: &PartialSolution) -> Option<PartialSolution> {
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
            Some(PartialSolution {
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
