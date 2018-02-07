pub struct Vocabulary<'r> {
    all: &'r [&'r str],
    by_length: Vec<Vec<&'r str>>
}

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
        Vocabulary {
            all: words,
            by_length
        }
    }

    pub fn with_length(&'r self, word_len: usize) -> Result<&'r [&'r str], String> {
        self.by_length.get(word_len).map(|v| v.as_slice()).ok_or(format!("No words with length {}", word_len))
    }

    pub fn all(&'r self) -> &'r [&'r str] {
        &self.all
    }
}
