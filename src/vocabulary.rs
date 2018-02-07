pub struct Vocabulary<'r> {
    all: &'r [&'r str],
    by_length: Vec<Vec<&'r str>>,
    statistics: Statistics
}

pub struct Statistics {
    pub length_distribution: LengthDistribution
}

pub struct LengthDistribution(Vec<usize>);

impl<'r> Vocabulary<'r> {
    pub fn new(words: &'r [&'r str]) -> Vocabulary<'r> {
        let mut by_length = Vec::new();
        for word in words {
            if by_length.len() < word.len() + 1 {
                for _ in 0 .. word.len() - by_length.len() + 1 {
                    by_length.push(Vec::new());
                }
            }
            by_length[word.len()].push(*word);
        }
        let length_distribution = by_length.iter().map(Vec::len).collect();
        Vocabulary {
            all: words,
            by_length,
            statistics: Statistics {
                length_distribution: LengthDistribution(length_distribution)
            }
        }
    }

    pub fn with_length(&'r self, word_len: usize) -> Result<&'r [&'r str], String> {
        self.by_length.get(word_len).map(|v| v.as_slice()).ok_or(format!("No words with length {}", word_len))
    }

    pub fn all(&'r self) -> &'r [&'r str] {
        &self.all
    }
}

impl LengthDistribution {
    pub fn get(&self, word_length: usize) -> Option<usize> {
        self.0.get(word_length).cloned()
    }
}
