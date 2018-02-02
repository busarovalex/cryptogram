#[derive(Debug)]
pub struct PatternWordIndex {
    current_indexes: Vec<usize>,
    lengths: Vec<usize>,
    first: bool,
    finished: bool
}

impl PatternWordIndex {
    pub fn new(lengths: Vec<usize>) -> PatternWordIndex {
        let current_indexes = vec![0; lengths.len()];
        PatternWordIndex {
            lengths,
            current_indexes,
            first: true,
            finished: false
        }
    }

    pub fn next(&mut self) -> Option<Vec<usize>> {
        if self.finished {
            return None;
        }
        if self.first {
            self.first = false;
            return Some(self.current_indexes.clone())
        }
        *self.current_indexes.last_mut().unwrap() += 1;
        
        self.increment();
        if self.finished {
            return None;
        }
        Some(self.current_indexes.clone())
    }

    pub fn increment_at(&mut self, index: usize) {

        for (index_number, index_value) in self.current_indexes.iter_mut().skip(index + 1).enumerate() {
            *index_value = self.lengths[index_number + index + 1] - 1;
        }
    }

    fn increment(&mut self) {
        let mut incremented_index = self.current_indexes.len() - 1;
        while self.current_indexes[incremented_index] == self.lengths[incremented_index] {
            self.current_indexes[incremented_index] = 0;
            if incremented_index == 0 {
                self.finished = true;
                break;
            }
            self.current_indexes[incremented_index - 1] += 1;
            incremented_index -= 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_next() {
        let mut index = PatternWordIndex::new(2, 3);

        assert_eq!(index.next(), Some(vec![0, 0]));
        assert_eq!(index.next(), Some(vec![0, 1]));
        assert_eq!(index.next(), Some(vec![0, 2]));

        assert_eq!(index.next(), Some(vec![1, 0]));
        assert_eq!(index.next(), Some(vec![1, 1]));
        assert_eq!(index.next(), Some(vec![1, 2]));

        assert_eq!(index.next(), Some(vec![2, 0]));
        assert_eq!(index.next(), Some(vec![2, 1]));
        assert_eq!(index.next(), Some(vec![2, 2]));

        assert_eq!(index.next(), None);        
    }

    #[test]
    fn correctly_increments() {
        let mut index = PatternWordIndex::new(3, 3);

        assert_eq!(index.next(), Some(vec![0, 0, 0]));
        assert_eq!(index.next(), Some(vec![0, 0, 1]));
        assert_eq!(index.next(), Some(vec![0, 0, 2]));
        assert_eq!(index.next(), Some(vec![0, 1, 0]));
        assert_eq!(index.next(), Some(vec![0, 1, 1]));

        index.increment_at(1);

        assert_eq!(index.next(), Some(vec![0, 2, 0]));
    }

    #[test]
    fn correctly_increments_2() {
        let mut index = PatternWordIndex::new(3, 3);

        assert_eq!(index.next(), Some(vec![0, 0, 0]));
        assert_eq!(index.next(), Some(vec![0, 0, 1]));

        index.increment_at(0);

        assert_eq!(index.next(), Some(vec![1, 0, 0]));
    }
}