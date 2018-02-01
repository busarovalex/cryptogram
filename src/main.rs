#![feature(underscore_lifetimes)]

extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use tokio_core::reactor::Core;
use telegram_bot::*;

use std::env;
use std::fs::File;
use std::io::prelude::*;

mod index;
mod pattern;
mod matches;

use index::{PatternWordIndex};
use pattern::{Pattern, PatternSystem};
use matches::{CombinedMatches};

const MAX_TOTAL_SATISFACTORY_MATCHES: usize = 200;
const MAX_TOTAL_WORD_TESTS: usize = 10_000_000;

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

        let (matches, message) = match find_words(&words, patterns) {
            Ok(result) => result,
            Err(error_message) => {
                println!("{}", &error_message);
                ::std::process::exit(1);
            }
        };

        if matches.is_empty() {
            println!("no results found!");
        } else {
            for combination in matches {
                println!("{}", combination);
            }
        }

        if let Some(message) = message {
            println!("{}", &message);
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
                    Ok((matches, info_message)) => {
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

                        if let Some(info_message) = info_message {
                            api.spawn(message.text_reply(info_message));
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

fn find_by_query(vocabulary: &[&str], query: &str) -> Result<(Vec<String>, Option<String>), String> {
    let patterns: Vec<String> = query.split_whitespace()
        .map(String::from)
        .collect();
    find_words(vocabulary, patterns)
}

fn find_words(vocabulary: &[&str], patterns_str: Vec<String>) -> Result<(Vec<String>, Option<String>), String> {
    if patterns_str.is_empty() {
        return Err(format!("No patters provided"));
    }

    let mut patterns: Vec<Pattern> = Vec::new();
    for pattern_str in &patterns_str {
        patterns.push(Pattern::new(pattern_str)?);
    }

    let pattern_system = PatternSystem::new(patterns.iter().collect())?;

    let orderd_patterns = pattern_system.ordered();

    let mut indexes = PatternWordIndex::new(orderd_patterns.len(), vocabulary.len());

    let mut satisfactory_matches: Vec<CombinedMatches> = Vec::new();

    let mut current_combined_match = CombinedMatches::empty();

    let mut total_satisfactory_matches = 0;
    let mut total_word_tests = 0;

    let mut too_many_matches_message = None;

    'outer: while let Some(ref matches_indexes) = indexes.next() {
        for (pattern_index, word_index) in matches_indexes.iter().enumerate() {

            let word: &str = &vocabulary[*word_index];
            let pattern = &orderd_patterns[pattern_index];
            if let Some(word_match) = pattern.match_word(word) {
                total_word_tests += 1;
                if total_word_tests > MAX_TOTAL_WORD_TESTS {
                    too_many_matches_message = Some(too_many_word_tests(&pattern_system));
                    break 'outer;
                }
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
        // println!("{:?}", current_combined_match);
        satisfactory_matches.push(::std::mem::replace(&mut current_combined_match, CombinedMatches::empty()));
        total_satisfactory_matches += 1;
        if total_satisfactory_matches > MAX_TOTAL_SATISFACTORY_MATCHES {
            return Err(too_many_satisfactory_results(&pattern_system));
        }
    }

    let mut result = Vec::new();

    for combined_match in satisfactory_matches {
        let mut match_set = String::new();
        for word in pattern_system.original_order(&combined_match.matches).iter().map(|m| m.word) {

            match_set.push_str(&word);
            match_set.push(' ');
        }
        result.push(match_set);
    }

    Ok((result, too_many_matches_message))
}

fn too_many_satisfactory_results(pattern_system: &PatternSystem<'_>) -> String {
    format!("Too many results. Probably, your patterns are not exact enough\n{}",
        pattern_sys_complexity_report(pattern_system))
}

fn too_many_word_tests(pattern_system: &PatternSystem<'_>) -> String {
    format!("Too many word tests. Probably, your patterns are not exact enough\n{}",
        pattern_sys_complexity_report(pattern_system))
}

fn pattern_sys_complexity_report(pattern_system: &PatternSystem<'_>) -> String {
    let mut sorted_patterns = String::new();

    for pattern in pattern_system.ordered()
        .iter()
        .map(|p| p.value) {
            sorted_patterns.push_str(pattern);
            sorted_patterns.push(' ');
    }

    let mut pattern_exactnesses = String::new();

    for pattern in pattern_system.patterns() {
        pattern_exactnesses.push_str(&format!("{} -> {}\n", pattern.value, pattern.exactness_score().0));
    }

    format!("patterns were sorted based on their exactness: {}\npattern exactnesses:\n{}pattern system exactness: {}", sorted_patterns, pattern_exactnesses, pattern_system.complexity_score().0)
}
