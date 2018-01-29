use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut file = File::open(::std::env::args().skip(1).next().unwrap()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let patterns: Vec<String> = ::std::env::args()
        .skip(2)
        .map(String::from)
        .collect();

    let mut groups: HashMap<WildcardsValues, HashMap<String, Vec<String>>> = HashMap::new();

    for word in contents.lines() {
        for pattern in &patterns {
            if let Some(wildcards_values) = test(word, pattern) {
                if groups.contains_key(&wildcards_values) {
                    groups.get_mut(&wildcards_values).unwrap()
                        .get_mut(pattern).unwrap()
                        .push(word.into());
                } else {
                    let pattern_map = patterns.iter()
                        .map(|p| if p == pattern 
                                    {(p.clone(), vec![word.into()])} 
                                    else 
                                    {(p.clone(), Vec::with_capacity(0))})
                        .collect();
                    groups.insert(wildcards_values, pattern_map);
                }
            }
        }
    }

    println!("{:?}", groups.len());

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

    println!("{:?}", combined_results.len());

    for (_, pattern_map) in combined_results {
        for matches in pattern_map.values() {
            for word in matches {
                print!("{} ", word);
            }
            println!("");    
        }
        println!("=================");
    }
}

fn test(word: &str, pattern: &str) -> Option<WildcardsValues> {
    if word.len() != pattern.len() {
        return None;
    }
    let mut wildcards_values = WildcardsValues::new();
    for (word_char, pattern_char) in word.chars().zip(pattern.chars()) {
        match pattern_char {
            '*' => {
                if wildcards_values.contains_word_char(word_char) {
                    return None;
                }
            },
            patter_char_value @ 'a' ... 'z' => {
                match wildcards_values.test_word_char(word_char, patter_char_value) {
                    WildcardValueResult::NotPresent => wildcards_values.add(word_char, patter_char_value),
                    WildcardValueResult::NotEqual => return None,
                    WildcardValueResult::Equal => {}
                }
            }
            unexpected @ _ => panic!("unexpected pattern char: {}", unexpected)
        }
    }
    Some(wildcards_values)
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
        assert!(test("zwitter", "**aa***").is_none());
        assert!(test("blooper", "**aa***").is_some());
        assert!(test("aabbaaa", "**aa***").is_some());
        assert!(test("aabba", "**aa***"  ).is_none());
    }

    #[test]
    fn test_wildcards_values() {
        println!("");
        let mut wildcards_values = WildcardsValues::new();
        assert_eq!(wildcards_values.test_word_char('a', 'b'), WildcardValueResult::NotPresent);
        wildcards_values.add('a', 'b');
        assert_eq!(wildcards_values.test_word_char('a', 'b'), WildcardValueResult::Equal);
        assert_eq!(wildcards_values.test_word_char('a', 'a'), WildcardValueResult::NotEqual);
        assert_eq!(wildcards_values.test_word_char('b', 'b'), WildcardValueResult::NotEqual);
    }
}
