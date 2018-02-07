use std::collections::{HashSet};
use std::ops::Index;
use std::ops::IndexMut;

use vocabulary::Vocabulary;

const UNKNOWN: char = '-';

#[derive(Debug)]
pub struct Cypher {
    phrase: Vec<String>,
    known_solution: Option<Solution>
}

#[derive(Debug)]
pub struct SolutionAggregator {
    cypher: Cypher,
    partial_solutions: Vec<PartialSolution>,
    full_soutions: Vec<Solution>
}

#[derive(Debug)]
struct PartialSolution {
    solution: Solution,
    satisfied_phrase_parts: HashSet<usize>
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct CharEncoding {
    from: char,
    to: char
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Solution {
    encoding: [char; 26]
}

#[derive(Debug)]
struct SolutionBuilder {
    encoding: [char; 26]
}

#[derive(Debug)]
pub enum Contradiction {
    Encoding(CharEncoding),
    CypherWordApplied(usize),
    InvalidChar(char)
}

impl SolutionAggregator {

    pub fn decode(&self) -> Vec<String> {
        let mut variants = Vec::new();
        for solution in &self.full_soutions {
            variants.push(self.cypher.decode_with(solution).join(" "));
        }
        variants
    }

    fn new(cypher: Cypher) -> SolutionAggregator {
        SolutionAggregator {
            cypher,
            partial_solutions: Vec::new(),
            full_soutions: Vec::new()
        }
    }

    fn visit(&mut self, word: &str) -> Result<(), Contradiction> {
        let phrase_len = self.cypher.phrase.len();
        for (cypher_word_index, cypher_word) in self.cypher.phrase.iter().enumerate() {
            if word.len() != cypher_word.len() { continue; }
            let mut solution_builder = match self.cypher.known_solution {
                Some(solution) => SolutionBuilder::based_on(solution),
                None => SolutionBuilder::new()
            };

            for (cypher_char, word_char) in cypher_word.chars().zip(word.chars()) {
                solution_builder.insert(CharEncoding::new(cypher_char, word_char))?;
            }

            let solution = solution_builder.build();

            if self.cypher.phrase.len() == 1 {
                self.full_soutions.push(solution);
            } else {
                self.partial_solutions.push(PartialSolution::new(solution, cypher_word_index));
            }

            let mut new_partial_solutions = Vec::new();

            for partial_solution in &self.partial_solutions {
                match partial_solution.add(&solution, cypher_word_index) {
                    Ok(new_partial_solution) => {
                        if new_partial_solution.satisfied_phrase_parts.len() == phrase_len {
                            self.full_soutions.push(new_partial_solution.solution);
                        } else {
                            new_partial_solutions.push(new_partial_solution);
                        }
                    },
                    Err(_) => {}
                }
            }

            self.partial_solutions.append(&mut new_partial_solutions);
        }
        Ok(())
    }
}

impl PartialSolution {
    fn new(solution: Solution, cypher_word_index: usize) -> PartialSolution {
        let mut satisfied_phrase_parts = HashSet::new();
        satisfied_phrase_parts.insert(cypher_word_index);
        PartialSolution {
            solution,
            satisfied_phrase_parts
        }
    }

    fn add(&self, new_solution: &Solution, cypher_word_index: usize) -> Result<PartialSolution, Contradiction> {
        if self.satisfied_phrase_parts.contains(&cypher_word_index) {
            return Err(Contradiction::same_cypher_word_already_applied(cypher_word_index));
        }
        let solution = SolutionBuilder::based_on(self.solution).add(*new_solution)?.build();
        let mut satisfied_phrase_parts = self.satisfied_phrase_parts.clone();
        satisfied_phrase_parts.insert(cypher_word_index);

        Ok(PartialSolution{
            solution,
            satisfied_phrase_parts
        })
    }
}

impl SolutionBuilder {
    fn new() -> SolutionBuilder {
        SolutionBuilder {
            encoding: [UNKNOWN; 26]
        }
    }

    fn based_on(base_solution: Solution) -> SolutionBuilder {
        SolutionBuilder {
            encoding: base_solution.encoding
        }
    }

    fn build(self) -> Solution {
        Solution {
            encoding: self.encoding
        }
    }

    fn add(mut self, other: Solution) -> Result<SolutionBuilder, Contradiction> {
        for (index, (other_char_encoding, self_char_encoding)) in other.encoding.into_iter().zip(self.encoding.iter_mut()).enumerate() {
            match (*other_char_encoding, self_char_encoding) {
                (UNKNOWN, _) => {},
                (new @ _, old @ &mut UNKNOWN) => *old = new,
                (new @ _, old @ &mut _) => if *old != new { return Err(contradiction(index, *other_char_encoding)) }
            }
        }
        Ok(self)
    }

    fn insert(&mut self, encoding: CharEncoding) -> Result<(), Contradiction> {
        let existing = self[encoding.from];
        if existing != UNKNOWN && existing != encoding.to {
            return Err(Contradiction::encoding(encoding.from, existing));
        }
        self[encoding.from] = encoding.to;
        Ok(())
    }
}

impl Index<char> for SolutionBuilder {
    type Output = char;
    fn index(&self, index: char) -> &Self::Output {
        &self.encoding[char_to_index(index)]
    }
}

impl Index<char> for Solution {
    type Output = char;
    fn index(&self, index: char) -> &Self::Output {
        &self.encoding[char_to_index(index)]
    }
}

impl IndexMut<char> for SolutionBuilder {
    fn index_mut(&mut self, index: char) -> &mut Self::Output {
        &mut self.encoding[char_to_index(index)]
    }
}

impl Cypher {
    pub fn new(phrase: Vec<String>, known_char_encodings: HashSet<CharEncoding>) -> Result<Cypher, Contradiction> {
        validate_phrase(&phrase)?;
        if known_char_encodings.is_empty() {
            return Ok(Cypher {
                phrase,
                known_solution: None
            })
        }
        
        let mut known_solution = SolutionBuilder::new();

        for char_encoding in known_char_encodings {
            known_solution.insert(char_encoding)?;
        }

        Ok(Cypher {
            phrase,
            known_solution: Some(known_solution.build())
        })
    }

    pub fn solve_for(self, vocabulary: &Vocabulary) -> SolutionAggregator {
        let mut aggregator = SolutionAggregator::new(self);
        for word in vocabulary.all() {
            match aggregator.visit(&word) {
                _ => {}
            }
        }
        aggregator
    }

    fn decode_with(&self, solution: &Solution) -> Vec<String> {
        let mut result = Vec::new();
        for word in &self.phrase {
            result.push(word.chars().map(|ch| solution[ch]).collect());
        }
        result
    }
}

impl Contradiction {
    fn encoding(from: char, to: char) -> Contradiction {
        Contradiction::Encoding(CharEncoding::new(from, to))
    }

    fn same_cypher_word_already_applied(cypher_word_index: usize) -> Contradiction {
        Contradiction::CypherWordApplied(cypher_word_index)
    }
}

impl CharEncoding {
    fn new(from: char, to: char) -> CharEncoding {
        CharEncoding {
            from,
            to
        }
    }
}

fn contradiction(index: usize, other_char_encoding: char) -> Contradiction {
    Contradiction::Encoding(CharEncoding::new(index_to_char(index), other_char_encoding))
}

fn char_to_index(ch: char) -> usize {
    (ch as u8 - 97) as usize
}

fn index_to_char(index: usize) -> char {
    (index as u8 + 97) as char
}

fn validate_phrase(phrase: &[String]) -> Result<(), Contradiction> {
    for word in phrase {
        for ch in word.chars() {
            match ch {
                'a' ... 'z' => {},
                invalid @ _ => return Err(Contradiction::InvalidChar(invalid))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vocabulary::Vocabulary;

    #[test]
    fn correctly_decripts() {
        assert_eq!(solutions("like", &["like"]), words(vec!["like"]));
        assert_eq!(solutions(
            "abc cba", 
            &[
                "like", "xyz", "zyx", "xyb", "zyb"
            ]), 
            words(vec!["zyx xyz", "xyz zyx"])
        );
    }

    #[test]
    fn test_char_to_index() {
        let chars = "abcdefghijklmnopqrstuvwxyz";
        assert_eq!(chars.len(), 26);
        for (index, ch) in chars.chars().enumerate() {
            assert_eq!(index, char_to_index(ch));
        }
    }

    #[test]
    fn test_index_to_char() {
        let chars = "abcdefghijklmnopqrstuvwxyz";
        assert_eq!(chars.len(), 26);
        for (index, ch) in chars.chars().enumerate() {
            assert_eq!(ch, index_to_char(index));
        }
    }

    fn words(original: Vec<&str>) -> Vec<String> {
        original.into_iter().map(|s| s.to_owned()).collect()
    }

    fn solutions(cypher: &str, vocabulary: &[&str]) -> Vec<String> {
        let vocabulary = Vocabulary::new(vocabulary);
        Cypher::new(
            cypher.split_whitespace().map(|s| s.to_owned()).collect(),
            HashSet::new()
        ).unwrap()
         .solve_for(&vocabulary)
         .decode()
    }
}