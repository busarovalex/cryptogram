use std::collections::{HashSet, HashMap};
use std::cmp::{Ord, PartialOrd, Ordering, Eq};

use matches::{Match, PlaceholderValues, WildcardValueResult};

const WILDCARD_COST: f32 = 1f32;
const WILDCARD_WORD_LENGTH_MULTIPLIER : f32 = 0.25 * 1.5f32;

const PLACEHOLDER_COST: f32 = 1f32;
const PLACEHOLDER_WORD_LENGTH_MULTIPLIER : f32 = 0.25 * 2f32;
const PLACEHOLDER_REPEAT_MULTIPLIER: f32 = 3f32;

const KNOWN_CHAR_COST: f32 = 2f32;
const KNOWN_CHAR_WORD_LENGTH_MULTIPLIER : f32 = 0.25 * 2f32;
const KNOWN_CHAR_REPEAT_MULTIPLIER: f32 = 4f32;

#[derive(Debug, Eq, PartialEq)]
pub struct Pattern<'r> {
    pub value: &'r str,
    pub known_chars: HashSet<char>
}

pub struct PatternSystem<'r> {
    patterns: Vec<&'r Pattern<'r>>,
    ordered_by_exactness: Vec<&'r Pattern<'r>>,
    order: Vec<usize>
}

#[derive(Debug, PartialEq)]
pub struct ExactnessScore(pub f32);

pub struct ComplexityScore(pub f32);

struct CharRepeat(HashMap<char, i32>);

fn validate_pattern_system<'r>(_: &PatternSystem<'r>) -> Result<(), String> {
    Ok(())
}

impl<'r> PatternSystem<'r> {
    pub fn new(patterns: Vec<&'r Pattern<'r>>) -> Result<PatternSystem<'r>, String> {
        let mut ordered_by_exactness = patterns.clone();

        ordered_by_exactness.sort_unstable_by_key(|p| p.exactness_score());

        let mut order = Vec::with_capacity(patterns.len());

        for pattern in &patterns {
            order.push(ordered_by_exactness.iter()
                .enumerate()
                .filter(|&(_, ref p)| ***p == **pattern)
                .map(|(i, _)| i)
                .next()
                .unwrap()
            )
        }

        let system = PatternSystem {
            patterns,
            ordered_by_exactness,
            order
        };

        validate_pattern_system(&system)?;

        Ok(system)
    }

    pub fn ordered(&self) -> &[&'r Pattern<'r>] {
        &self.ordered_by_exactness
    }

    pub fn complexity_score(&self) -> ComplexityScore {
        let mut score = 0f32;
        for pattern in &self.patterns {
            score += pattern.exactness_score().0;
        }
        ComplexityScore(100. / score)
    }

    pub fn original_order<'k, 'a, 'b>(&self, exactness_order: &'k [Match<'a, 'b>]) -> Vec<&'k Match<'a, 'b>> {
        let mut original_order = Vec::with_capacity(exactness_order.len());
        for index_in_ordered in &self.order {
            original_order.push(&exactness_order[*index_in_ordered]);
        }
        original_order
    }
}

impl<'r> Pattern<'r> {
    pub fn new(value: &'r str) -> Result<Pattern<'r>, String> {
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

    pub fn match_word<'a>(&'r self, word: &'a str) -> Option<Match<'r, 'a>> {
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

    pub fn exactness_score(&self) -> ExactnessScore {
        let mut score = 0f32;
        let word_length = self.value.len() as f32;
        let mut known_char_repeat = CharRepeat::new();
        let mut placeholder_repeat = CharRepeat::new();

        let repeat_mul = |num, mul| {
            if num == 0 {
                1f32
            } else {
                num as f32 * mul
            }
        };

        for ch in self.value.chars() {
            match ch {
                '*' | '_' | '+' => score += WILDCARD_COST * WILDCARD_WORD_LENGTH_MULTIPLIER * word_length,
                'a' ... 'z' => score += KNOWN_CHAR_COST * KNOWN_CHAR_WORD_LENGTH_MULTIPLIER * word_length * repeat_mul(known_char_repeat.repeats(ch), KNOWN_CHAR_REPEAT_MULTIPLIER),
                ch @ '0' ... '9' => score += PLACEHOLDER_COST * PLACEHOLDER_WORD_LENGTH_MULTIPLIER * word_length * repeat_mul(placeholder_repeat.repeats(ch),PLACEHOLDER_REPEAT_MULTIPLIER),
                _ => unreachable!()
            }
        }
        ExactnessScore(score)
    }
}

impl CharRepeat {
    fn new() -> CharRepeat {
        CharRepeat(HashMap::new())
    }

    fn repeats(&mut self, ch: char) -> i32 {
        if let Some(count) = self.0.get_mut(&ch) {
            *count += 1;
            return *count;
        }
        self.0.insert(ch, 1);
        1
    }
}

impl Ord for ExactnessScore {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for ExactnessScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ExactnessScore {

}
