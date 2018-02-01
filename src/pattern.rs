use std::collections::{HashSet};

use matches::{Match, PlaceholderValues, WildcardValueResult};

#[derive(Debug)]
pub struct Pattern<'r> {
    pub value: &'r str,
    pub known_chars: HashSet<char>
}

impl<'r> Pattern<'r> {
    pub fn new(value: &'r str) -> Result<Pattern<'r>, String> {
        if value.chars()
            .any(|ch| match ch { '*' | '+' | '_' | '0' ... '9' | 'a' ... 'z' => false, _ => true }) {
                return Err(format!("pattern {} has invalid characters", value));
        }
        Ok(Pattern {
            value,
            known_chars: value.chars()
                              .filter(|ch| match *ch {'a' ... 'z' => true, _ => false})
                              .collect()
        })
    }

    pub fn match_word<'a>(&'r self, word: &'a str) -> Option<Match<'r, 'a>> {
        if word.len() != self.value.len() {
            return None;
        }
        let mut placeholder_values = PlaceholderValues::new();
        let mut wildcard_values = HashSet::new();
        let known_chars = &self.known_chars;
        for (word_char, pattern_char) in word.chars().zip(self.value.chars()) {
            match pattern_char {
                '*' | '+' | '_' => {
                    if placeholder_values.contains_word_char(word_char) {
                        return None;
                    }
                    if known_chars.contains(&word_char) {
                        return None;
                    }
                    if wildcard_values.contains(&word_char) {
                        return None;
                    } else {
                        wildcard_values.insert(word_char);
                    }
                },
                known_char_value @ 'a' ... 'z' => {
                    if word_char != known_char_value {
                        return None;
                    }
                },
                patter_char_value @ '0' ... '9' => {
                    if known_chars.contains(&word_char) {
                        return None;
                    }
                    match placeholder_values.test_word_char(word_char, patter_char_value) {
                        WildcardValueResult::NotPresent => placeholder_values.add(word_char, patter_char_value),
                        WildcardValueResult::NotEqual => return None,
                        WildcardValueResult::Equal => {}
                    }
                },
                unexpected @ _ => unreachable!("unexpected char: {}", unexpected)
            }
        }
        Some(Match {
            pattern: &self,
            word,
            wildcard_values,
            placeholder_values
        })
    }
}
