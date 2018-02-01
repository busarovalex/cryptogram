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

    let pattern_system = PatternSystem::new(patterns.iter().collect())?;

    let orderd_patterns = pattern_system.ordered();

    let mut indexes = PatternWordIndex::new(orderd_patterns.len(), vocabulary.len());

    let mut satisfactory_matches: Vec<CombinedMatches> = Vec::new();

    let mut current_combined_match = CombinedMatches::empty();

    'outer: while let Some(ref matches_indexes) = indexes.next() {
        for (pattern_index, word_index) in matches_indexes.iter().enumerate() {

            let word: &str = &vocabulary[*word_index];
            let pattern = &orderd_patterns[pattern_index];
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
        for word in pattern_system.original_order(&combined_match.matches).iter().map(|m| m.word) {

            match_set.push_str(&word);
            match_set.push(' ');
        }
        result.push(match_set);
    }

    Ok(result)
}
