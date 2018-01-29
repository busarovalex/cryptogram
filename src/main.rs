extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;

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

fn find_words(vocabulary: &[&str], patterns: Vec<String>) -> Result<Vec<String>, String> {
    let mut groups: HashMap<WildcardsValues, HashMap<String, Vec<String>>> = HashMap::new();

    let known_chars_map: HashMap<String, HashSet<char>> = patterns.iter()
        .map(|pattern| (
            pattern.to_string(),
            pattern.chars().filter(|ch| match *ch {'a' ... 'z' => true, _ => false}).collect()
        ))
        .collect();

    for word in vocabulary {
        for pattern in &patterns {
            let pattern_known_chars = known_chars_map.get(pattern).unwrap();
            if let Some(wildcards_values) = test(word, pattern, pattern_known_chars)? {
                if groups.contains_key(&wildcards_values) {
                    groups.get_mut(&wildcards_values).unwrap()
                        .get_mut(pattern).unwrap()
                        .push(word.to_string());
                } else {
                    let pattern_map = patterns.iter()
                        .map(|p| if p == pattern 
                                    {(p.clone(), vec![word.to_string()])} 
                                    else 
                                    {(p.clone(), Vec::with_capacity(0))})
                        .collect();
                    groups.insert(wildcards_values, pattern_map);
                }
            }
        }
    }

    let mut combined_results: Vec<(WildcardsValues, HashMap<String, Vec<String>>)> = Vec::new();

    for (wildcards_values, pattern_map) in groups {
        for &mut (ref mut combined_wildcards_values, ref mut combined_pattern_map) in &mut combined_results {
            if combined_wildcards_values.does_not_contradict_with(&wildcards_values) {
                *combined_wildcards_values = combined_wildcards_values.merge(&wildcards_values);
                for (pattern, matched_values) in combined_pattern_map.iter_mut() {
                    let mut cloned_matched_values = pattern_map.get(pattern).unwrap().clone();
                    matched_values.append(&mut cloned_matched_values);
                }
            }
        }
        combined_results.push((wildcards_values, pattern_map));
    }

    combined_results.retain(
        |&(_, ref pattern_map)| 
        !pattern_map.values().any(Vec::is_empty)
    );

    let mut result = Vec::new();

    for (_, pattern_map) in combined_results {
        let mut wildcard_combination_result = String::new();
        for matches in pattern_map.values() {
            for word in matches {
                wildcard_combination_result.push_str(word);
                wildcard_combination_result.push(' ');
            }
            wildcard_combination_result.push('\n');
        }
        wildcard_combination_result.push_str("=================\n");
        result.push(wildcard_combination_result);
    }

    Ok(result)
}

fn test(word: &str, pattern: &str, known_chars: &HashSet<char>) -> Result<Option<WildcardsValues>, String> {
    if word.len() != pattern.len() {
        return Ok(None);
    }
    let mut wildcards_values = WildcardsValues::new();
    for (word_char, pattern_char) in word.chars().zip(pattern.chars()) {
        match pattern_char {
            '*' | '+' | '_' => {
                if wildcards_values.contains_word_char(word_char) {
                    return Ok(None);
                }
                if known_chars.contains(&word_char) {
                    return Ok(None);
                }
            },
            known_char_value @ 'a' ... 'z' => {
                if word_char != known_char_value {
                    return Ok(None)
                }
            },
            patter_char_value @ '0' ... '9' => {
                if known_chars.contains(&word_char) {
                    return Ok(None);
                }
                match wildcards_values.test_word_char(word_char, patter_char_value) {
                    WildcardValueResult::NotPresent => wildcards_values.add(word_char, patter_char_value),
                    WildcardValueResult::NotEqual => return Ok(None),
                    WildcardValueResult::Equal => {}
                }
            }
            unexpected @ _ => return Err(format!("unexpected pattern char: {}", unexpected))
        }
    }
    Ok(Some(wildcards_values))
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct WildcardsValues {
    values: Vec<(char, char)>
}

#[derive(Debug, Eq, PartialEq)]
enum WildcardValueResult {
    NotPresent,
    NotEqual,
    Equal
}

impl WildcardsValues {
    fn new() -> WildcardsValues {
        WildcardsValues {
            values: Vec::with_capacity(0)
        }
    }

    fn add(&mut self, word_char: char, pattern_char_value: char) {
        self.values.push((word_char, pattern_char_value));
    }

    fn test_word_char(&self, word_char: char, pattern_char_value: char) -> WildcardValueResult {
        for &(existing_word_char, existing_pattern_char_value) in self.values.iter() {
            #[cfg(test)]
            {
                println!(
                    "({}, {}) -- ({}, {})", 
                    word_char, pattern_char_value, 
                    existing_word_char, existing_pattern_char_value
                );
            }
            if word_char == existing_word_char && pattern_char_value == existing_pattern_char_value {
                return WildcardValueResult::Equal;
            }
            if (word_char == existing_word_char && pattern_char_value != existing_pattern_char_value) ||
               (word_char != existing_word_char && pattern_char_value == existing_pattern_char_value) {
                return WildcardValueResult::NotEqual;
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

    fn does_not_contradict_with(&self, other: &WildcardsValues) -> bool {
        for &(word_char, pattern_char) in &self.values {
            for &(other_word_char, other_pattern_char) in &other.values {
                match (word_char == other_word_char, pattern_char == other_pattern_char) {
                    (true, false) | (false, true) => return false,
                    _ => {}
                }
            }
        }
        true
    }

    fn merge(&self, other: &WildcardsValues) -> WildcardsValues {
        let mut new_values = self.values.clone();

        for &(word_char, pattern_char) in &self.values {
            for &(other_word_char, other_pattern_char) in &other.values {
                if word_char != other_word_char || pattern_char != other_pattern_char {
                    new_values.push((other_word_char, other_pattern_char));
                }
           }
        }

        new_values.dedup_by(|&mut (a, b), &mut (c, d)| a == c && b == d);

        WildcardsValues {
            values: new_values
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_match() {
        let empty_map = HashSet::new();
        assert!(test("zwitter", "**11***", &empty_map).unwrap().is_none());
        assert!(test("blooper", "**11***", &empty_map).unwrap().is_some());
        assert!(test("aabbaaa", "**11***", &empty_map).unwrap().is_some());
        assert!(test("aabba",   "**11***", &empty_map).unwrap().is_none());
    }

    #[test]
    fn test_wildcards_values() {
        println!("");
        let mut wildcards_values = WildcardsValues::new();
        assert_eq!(wildcards_values.test_word_char('a', '1'), WildcardValueResult::NotPresent);
        wildcards_values.add('a', '1');
        assert_eq!(wildcards_values.test_word_char('a', '1'), WildcardValueResult::Equal);
        assert_eq!(wildcards_values.test_word_char('a', '2'), WildcardValueResult::NotEqual);
        assert_eq!(wildcards_values.test_word_char('b', '1'), WildcardValueResult::NotEqual);
    }

    #[test]
    fn test_pattern_match_with_known_values() {
        let mut set_of_known_values = HashSet::new();
        set_of_known_values.insert('e');

        assert!(test("wellness", "+++1+e22", &set_of_known_values).unwrap().is_none());
    }

    //
}
