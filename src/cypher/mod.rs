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
    full_soutions: HashSet<Solution>
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

#[derive(Debug, Clone, Copy)]
struct SolutionBuilder {
    encoding: [char; 26]
}

#[derive(Debug)]
pub enum Contradiction {
    Encoding(CharEncoding),
    // CypherWordApplied(usize),
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
            full_soutions: HashSet::new()
        }
    }

    fn solve_for(&mut self, vocabulary: &Vocabulary) {
        let mut first = true;
        for cypher_word in &self.cypher.phrase {
            println!("now searching matches for {}", cypher_word);
            let new_solutions = self.solve_for_cypher_word(vocabulary, cypher_word, first);
            println!("possible solutions found: {}", new_solutions.len());
            self.full_soutions = new_solutions;
            first = false;
        }
    }

    fn solve_for_cypher_word(&self, vocabulary: &Vocabulary, cypher_word: &str, first: bool) -> HashSet<Solution> {
        let cypher_word_len = cypher_word.len();
        let mut new_solutions = HashSet::with_capacity(vocabulary.all().len());
        let initial_solution = self.cypher.known_solution.unwrap_or(Solution::new());
        'outer: for word in vocabulary.all() {
            if word.len() != cypher_word_len { continue; }
            let mut solution_builder = SolutionBuilder::based_on(initial_solution);

            for (cypher_char, word_char) in cypher_word.chars().zip(word.chars()) {
                match solution_builder.insert(CharEncoding::new(cypher_char, word_char)) {
                    Ok(_) => {},
                    Err(_) => continue 'outer
                }
            }

            if first {
                new_solutions.insert(solution_builder.build());
                continue 'outer;
            }

            'inner: for already_existing_solution in &self.full_soutions {
                match solution_builder.add(*already_existing_solution) {
                    Ok(combined_solution) => { new_solutions.insert(combined_solution.build()); },
                    Err(_) => continue 'inner
                }
            }
        }
        new_solutions
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
        #[cfg(feature="unsafe")]
        {
            unsafe {
                self.encoding.get_unchecked(char_to_index(index))    
            }
        }
        #[cfg(not(feature="unsafe"))]
        {
            &self.encoding[char_to_index(index)]    
        }
    }
}

impl Index<char> for Solution {
    type Output = char;
    fn index(&self, index: char) -> &Self::Output {
        #[cfg(feature="unsafe")]
        {
            unsafe {
                self.encoding.get_unchecked(char_to_index(index))    
            }
        }
        #[cfg(not(feature="unsafe"))]
        {
            &self.encoding[char_to_index(index)]    
        }
    }
}

impl IndexMut<char> for SolutionBuilder {
    fn index_mut(&mut self, index: char) -> &mut Self::Output {
        #[cfg(feature="unsafe")]
        {
            unsafe {
                self.encoding.get_unchecked_mut(char_to_index(index))    
            }
        }
        #[cfg(not(feature="unsafe"))]
        {
            &mut self.encoding[char_to_index(index)]    
        }
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
        aggregator.solve_for(vocabulary);
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

impl Solution {
    fn new() -> Solution {
        SolutionBuilder::new().build()
    }
}

impl Contradiction {
    fn encoding(from: char, to: char) -> Contradiction {
        Contradiction::Encoding(CharEncoding::new(from, to))
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
            words(vec!["xyz zyx", "zyx xyz"])
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