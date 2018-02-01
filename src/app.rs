#[derive(StructOpt, Debug)]
#[structopt(name = "CryptoFind", about = "Finds words for 'Cryptogram'")]
pub struct App {
    #[structopt(help = "Vocabulary file")]
    pub vocabulary: String,

    #[structopt(help = "List of patterns")]
    pub patterns: Vec<String>
}