extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;

use std::collections::{HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;

mod index;

use index::{PatternWordIndex};

fn main() {
    if let Some(vocabulary_name) = ::std::env::args().skip(1).next() {
        let mut file = File::open(vocabulary_name).unwrap();
        let mut vocabulary = String::new();
        file.read_to_string(&mut vocabulary).unwrap();

        let words: Vec<_> = vocabulary.lines()
            .collect();

        let patterns: Vec<_> = ::std::env::args().skip(2)
            .map(String::from)
            .collect();

        let result = match find_words(&words, patterns) {
            Ok(matches) => matches,
            Err(error_message) => {
                println!("{}", &error_message);
                ::std::process::exit(1);
            }
        };

        if result.is_empty() {
            println!("no results found!");
        } else {
            for combination in result {
                println!("{}", combination);
            }
        }
        
        return;
    }

    let mut core = Core::new().unwrap();

    let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    let vocabulary = include_str!("../10kwords.txt");
    let words: Vec<_> = vocabulary.lines()
        .collect();


    // Fetch new updates via long poll method
    let future = api.stream().for_each(|update| {

        // If the received update contains a new message...
        if let UpdateKind::Message(message) = update.kind {

            if let MessageKind::Text {ref data, ..} = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                match find_by_query(&words, data) {
                    Ok(matches) => {
                        if matches.is_empty() {
                            api.spawn(message.text_reply(format!("No results found!")));
                        } else {
                            let mut chunk = String::new();
                            for single_match in matches {
                                if (chunk.len() + single_match.len()) > 4000 {
                                    api.spawn(message.text_reply(::std::mem::replace(&mut chunk, String::new())));
                                }
                                chunk.push_str(&single_match);
                                chunk.push('\n');
                            }
                            api.spawn(message.text_reply(chunk));
                        }
                    },
                    Err(error_message) => api.spawn(message.text_reply(error_message))
                }
                
            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}

fn find_by_query(vocabulary: &[&str], query: &str) -> Result<Vec<String>, String> {
    let patterns: Vec<String> = query.split_whitespace()
        .map(String::from)
        .collect();
    find_words(vocabulary, patterns)
}

fn find_words(vocabulary: &[&str], patterns_str: Vec<String>) -> Result<Vec<String>, String> {
    if patterns_str.is_empty() {
        return Err(format!("No patters provided"));
    }

    let mut patterns: Vec<Pattern> = Vec::new();
    for pattern_str in &patterns_str {
        patterns.push(Pattern::new(pattern_str)?);
    }

    let mut indexes = PatternWordIndex::new(patterns.len(), vocabulary.len());

    let mut satisfactory_matches: Vec<CombinedMatches> = Vec::new();

    let mut current_combined_match = CombinedMatches::empty();

    'outer: while let Some(ref matches_indexes) = indexes.next() {
        for (pattern_index, word_index) in matches_indexes.iter().enumerate() {

            let word: &str = &vocabulary[*word_index];
            let pattern = &patterns[pattern_index];
            if let Some(word_match) = pattern.match_word(word) {
                if current_combined_match.contradicts_with(&word_match) {
                    current_combined_match = CombinedMatches::empty();
                    indexes.increment_at(pattern_index);
                    continue 'outer;
                } else {
                    current_combined_match.add(word_match);
                }
            } else {
                current_combined_match = CombinedMatches::empty();
                indexes.increment_at(pattern_index);
                continue 'outer;
            }
        }
        satisfactory_matches.push(::std::mem::replace(&mut current_combined_match, CombinedMatches::empty()));
    }

    let mut result = Vec::new();

    for combined_match in satisfactory_matches {
        let mut match_set = String::new();
        for word in combined_match.matches.iter().map(|m| m.word) {
            match_set.push_str(&word);
            match_set.push(' ');
        }
        result.push(match_set);
    }

    Ok(result)
}

#[derive(Debug)]
struct Pattern<'r> {
    value: &'r str,
    known_chars: HashSet<char>
}

#[derive(Debug)]
struct Match<'a, 'b> {
    pattern: &'a Pattern<'a>,
    word: &'b str,
    wildcard_values: HashSet<char>,
    placeholder_values: PlaceholderValues
}

#[derive(Debug)]
struct CombinedMatches<'a, 'b> {
    matches: Vec<Match<'a, 'b>>,
    wildcard_values: HashSet<char>,
    placeholder_values: PlaceholderValues
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct PlaceholderValues {
    values: Vec<(char, char)>
}

#[derive(Debug, Eq, PartialEq)]
enum WildcardValueResult {
    NotPresent,
    NotEqual,
    Equal
}

impl<'a, 'b> CombinedMatches<'a, 'b> {
    fn empty() -> CombinedMatches<'a, 'b> {
        CombinedMatches {
            matches: Vec::with_capacity(2),
            wildcard_values: HashSet::new(),
            placeholder_values: PlaceholderValues::new()
        }
    }

    fn contradicts_with(&self, other: &Match) -> bool {
        // println!("{:?} contradicts_with {:?}", &self, other);
        if !self.wildcard_values.is_disjoint(&other.wildcard_values) {
            // println!("      false");
            return true;
        }
        for word_match in &self.matches {
            if !word_match.pattern.known_chars.is_disjoint(&other.wildcard_values) {
                // println!("{:?}: {:?}", &word_match.pattern.known_chars, &other.wildcard_values);
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

    fn add(&mut self, other: Match<'a, 'b>) {
        for wildcard_value in &other.wildcard_values {
            self.wildcard_values.insert(*wildcard_value);
        }
        self.placeholder_values = self.placeholder_values.merge(&other.placeholder_values);
        self.matches.push(other);
    }
}

impl<'r> Pattern<'r> {
    fn new(value: &'r str) -> Result<Pattern<'r>, String> {
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

    fn match_word<'a>(&'r self, word: &'a str) -> Option<Match<'r, 'a>> {
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

impl PlaceholderValues {
    fn new() -> PlaceholderValues {
        PlaceholderValues {
            values: Vec::with_capacity(0)
        }
    }

    fn add(&mut self, word_char: char, pattern_char_value: char) {
        self.values.push((word_char, pattern_char_value));
    }

    fn test_word_char(&self, word_char: char, pattern_char_value: char) -> WildcardValueResult {
        for &(existing_word_char, existing_pattern_char_value) in self.values.iter() {
            match (word_char == existing_word_char, pattern_char_value == existing_pattern_char_value) {
                (true, true) => return WildcardValueResult::Equal,
                (true, false) | (false, true) => return WildcardValueResult::NotEqual,
                _ => {}
            }
        }
        WildcardValueResult::NotPresent
    }

    fn contains_word_char(&self, word_char: char) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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

    fn pattern(value: &'static str) -> Pattern {
        Pattern::new(value).unwrap()
    }
}
