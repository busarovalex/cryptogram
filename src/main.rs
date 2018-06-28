#![feature(underscore_lifetimes)]
#![feature(inclusive_range_syntax)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

use std::fs::File;
use std::io::{self, Read};

mod app;
mod vocabulary;
mod vocabulary_index;
mod cipher_text;
mod decipher;
mod render;

use app::App;
use vocabulary::Vocabulary;
use vocabulary_index::VocabularyIndex;
use cipher_text::CipherText;
use decipher::Decipher;
use render::Render;

fn main() {
    env_logger::init();
    let app = App::from_args();

    if app.chipher_text.is_empty() {
        println!("No cipher provided!");
        return;
    }

    let vocabulary_name = app.vocabulary;
    let mut file = File::open(vocabulary_name).unwrap();
    let mut vocabulary = String::new();
    file.read_to_string(&mut vocabulary).unwrap();

    let words: Vec<_> = vocabulary.lines().collect();

    let vocabulary = Vocabulary::new(&words);
    debug!("{:?}", vocabulary);

    let index = VocabularyIndex::new(&vocabulary);
    debug!("{:#?}", index);

    let mut cipher_text = CipherText::new(app.chipher_text);
    debug!("{:#?}", &cipher_text);

    println!("Current conditions: {}", &cipher_text);
    println!("Reorder?");
    if let Some(reorder) = reorder() {
        cipher_text.reorder_conditions(&reorder);
        println!("Reordered: {}", &cipher_text);
    }

    let solution = Decipher::new(index, &cipher_text).find_solution();
    debug!("{:?}", solution);

    let render = Render::new(solution, &vocabulary, &cipher_text);
    println!("{}", render);
}

fn reorder() -> Option<Vec<usize>> {
    let mut pattern = String::new();
    io::stdin().read_line(&mut pattern).unwrap();
    if pattern.is_empty() || &pattern == "\n" {
        return None;
    }

    let indexes: Vec<usize> = pattern
        .split_whitespace()
        .map(|index| index.parse().unwrap())
        .collect();

    Some(indexes)
}
