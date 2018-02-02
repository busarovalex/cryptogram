use std::collections::{HashSet};

use pattern::{Pattern};

#[derive(Debug)]
pub struct Match<'a, 'b> {
    pub pattern: &'a Pattern<'a>,
    pub word: &'b str,
    pub wildcard_values: HashSet<char>,
    pub placeholder_values: PlaceholderValues
}

#[derive(Debug)]
pub struct CombinedMatches<'a, 'b> {
    pub matches: Vec<Match<'a, 'b>>,
    pub wildcard_values: HashSet<char>,
    pub placeholder_values: PlaceholderValues
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PlaceholderValues {
    pub values: Vec<(char, char)>
}

#[derive(Debug, Eq, PartialEq)]
pub enum WildcardValueResult {
    NotPresent,
    NotEqual,
    Equal
}

impl<'a, 'b> CombinedMatches<'a, 'b> {
    pub fn empty() -> CombinedMatches<'a, 'b> {
        CombinedMatches {
            matches: Vec::with_capacity(2),
            wildcard_values: HashSet::new(),
            placeholder_values: PlaceholderValues::new()
        }
    }

    pub fn contradicts_with(&self, other: &Match) -> bool {
        if !self.wildcard_values.is_disjoint(&other.wildcard_values) {
            return true;
        }
        for word_match in &self.matches {
            if !word_match.pattern.known_chars.is_disjoint(&other.wildcard_values) {
                return true;
            }

            for known_char in &word_match.pattern.known_chars {
                if other.placeholder_values.contains_word_char(*known_char) {
                    return true;
                }
            }
        }
        self.placeholder_values.contradicts_with(&other.placeholder_values)

    }

    pub fn add(&mut self, other: Match<'a, 'b>) {
        for wildcard_value in &other.wildcard_values {
            self.wildcard_values.insert(*wildcard_value);
        }
        self.placeholder_values = self.placeholder_values.merge(&other.placeholder_values);
        self.matches.push(other);
    }
}

impl PlaceholderValues {
    pub fn new() -> PlaceholderValues {
        PlaceholderValues {
            values: Vec::with_capacity(0)
        }
    }

    pub fn add(&mut self, word_char: char, pattern_char_value: char) {
        self.values.push((word_char, pattern_char_value));
    }

    pub fn test_word_char(&self, word_char: char, pattern_char_value: char) -> WildcardValueResult {
        for &(existing_word_char, existing_pattern_char_value) in self.values.iter() {
            match (word_char == existing_word_char, pattern_char_value == existing_pattern_char_value) {
                (true, true) => return WildcardValueResult::Equal,
                (true, false) | (false, true) => return WildcardValueResult::NotEqual,
                _ => {}
            }
        }
        WildcardValueResult::NotPresent
    }

    pub fn contains_word_char(&self, word_char: char) -> bool {
        for &(existing_word_char, _) in self.values.iter() {
            if existing_word_char == word_char {
                return true;
            }
        }
        false
    }

    fn contradicts_with(&self, other: &PlaceholderValues) -> bool {
        for &(word_char, pattern_char) in &self.values {
            for &(other_word_char, other_pattern_char) in &other.values {
                match (word_char == other_word_char, pattern_char == other_pattern_char) {
                    (true, false) | (false, true) => return true,
                    _ => {}
                }
            }
        }
        false
    }

    fn merge(&self, other: &PlaceholderValues) -> PlaceholderValues {
        if self.values.is_empty() {
            return PlaceholderValues {
                values: other.values.clone()
            }
        }

        let mut new_values: HashSet<_> = self.values.iter().cloned().collect();

        for &(word_char, pattern_char) in &self.values {
            for &(other_word_char, other_pattern_char) in &other.values {
                if word_char != other_word_char || pattern_char != other_pattern_char {
                    new_values.insert((other_word_char, other_pattern_char));
                }
           }
        }

        PlaceholderValues {
            values: new_values.into_iter().collect()
        }
    }
}


impl<'a, 'b> Match<'a, 'b> {
    pub fn contradicts_pattern(&self, pattern: &Pattern) -> bool {
        for ch in pattern.known_chars.iter() {
            if self.wildcard_values.contains(&ch) {
                return true;
            }
            if self.placeholder_values.contains_word_char(*ch) {
                return true;
            }
        }
        false
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pattern::{Pattern};

    #[test]
    fn test_pattern_match() {
        assert!(pattern("**11***").match_word("zwitter").is_none());
        assert!(pattern("**11***").match_word("blooper").is_some());
        assert!(pattern("**11***").match_word("abccdef").is_some());
        assert!(pattern("**11***").match_word("aabba")  .is_none());
    }

    #[test]
    fn test_placeholder_values() {
        let mut placeholder_values = PlaceholderValues::new();
        assert_eq!(placeholder_values.test_word_char('a', '1'), WildcardValueResult::NotPresent);
        placeholder_values.add('a', '1');
        assert_eq!(placeholder_values.test_word_char('a', '1'), WildcardValueResult::Equal);
        assert_eq!(placeholder_values.test_word_char('a', '2'), WildcardValueResult::NotEqual);
        assert_eq!(placeholder_values.test_word_char('b', '1'), WildcardValueResult::NotEqual);
    }

    #[test]
    fn test_pattern_match_with_known_values() {
        assert!(pattern("+++1+e22").match_word("wellness").is_none());
        assert!(pattern("any+n_").match_word("anyone").is_some());
    }

    #[test]
    fn does_not_match_word_with_repeated_chars() {
        assert!(pattern("++").match_word("ee").is_none());
        assert!(pattern("1+").match_word("ee").is_none());
    }

    #[test]
    fn test_disjoint() {
        let mut set = HashSet::new();
        set.insert('a');
        assert!(!set.is_disjoint(&set));
    }

    #[test]
    fn contradicts_with_pattern() {
        let initial_pattern = pattern("++1");
        let word_match = initial_pattern.match_word("abс").unwrap();

        assert!(word_match.contradicts_pattern(&pattern("a+")));
        assert!(word_match.contradicts_pattern(&pattern("с1")));
        assert!(!word_match.contradicts_pattern(&pattern("++")));
    }

    fn pattern(value: &'static str) -> Pattern {
        Pattern::new(value).unwrap()
    }
}
