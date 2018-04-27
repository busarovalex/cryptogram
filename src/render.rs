use std::fmt;

use vocabulary::Vocabulary;
use decipher::{Solution};
use cipher_text::{CipherText, CipherWordId};

pub struct Render<'r, 'a> {
    solution: Solution,
    vocabulary: &'r Vocabulary<'r>,
    cipher: &'a CipherText
}

struct SolutionsForSingleWord<'r> {
    cipher_word_id: CipherWordId,
    length: usize,
    empty: String,
    words: Vec<&'r str>,
}

impl<'r, 'a> Render<'r, 'a> {
    pub fn new(solution: Solution,
    vocabulary: &'r Vocabulary,
    cipher: &'a CipherText) -> Render<'r, 'a> {
        Render {
            solution,
            vocabulary,
            cipher
        }
    }

    fn render(&self) -> String {
        let mut rendered = String::with_capacity(4 * 1024);
        for partial_solution in self.solution.partial_soultions() {
            let mut solutions = Vec::with_capacity(10);
            for (cipher_word_id, words) in partial_solution.satisfactory_words() {
                let mut for_word = SolutionsForSingleWord::new(
                    *cipher_word_id, 
                    words.len(),
                    self.cipher.length_of(*cipher_word_id).unwrap()
                );

                for word_id in words.ids() {
                    let word = self.vocabulary.get(*word_id).unwrap();
                    for_word.add(word);
                }

                solutions.push(for_word);
            }
            solutions.sort_by_key(|s| s.cipher_word_id);

            let max_number_of_words = solutions.iter()
                .map(|s| s.words.len())
                .max()
                .unwrap();

            for solution_word_index in 0..max_number_of_words {
                for by_cipher_word in &solutions {
                    let word = by_cipher_word.words.get(solution_word_index)
                        .map(|w| *w)
                        .unwrap_or(by_cipher_word.empty.as_str());
                    rendered.push_str(word);
                rendered.push_str(", ");
                }
                rendered.push('\n');
            }
            if max_number_of_words > 1 {
                rendered.push_str("\n");
            }
        }
        rendered
    }
}

impl<'r> SolutionsForSingleWord<'r> {
    fn new(cipher_word_id: CipherWordId, capacity: usize, length: usize,) -> SolutionsForSingleWord<'r> {
        let mut empty = String::with_capacity(length);
        for _ in 0..length {
            empty.push(' ');
        }
        SolutionsForSingleWord {
            cipher_word_id,
            length,
            empty,
            words: Vec::with_capacity(capacity)
        }
    }

    fn add(&mut self, word: &'r str) {
        self.words.push(word);
    }
}

impl<'r, 'a> fmt::Display for Render<'r, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.render())
    }
}