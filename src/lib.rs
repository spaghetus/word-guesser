use std::collections::BTreeMap;

use rayon::prelude::*;

#[cfg(feature = "dictionary")]
const WORDS_FILE: &str = include_str!("words.list");

#[cfg(feature = "dictionary")]
lazy_static::lazy_static! {
	pub static ref WORDS: Vec<&'static str> = WORDS_FILE.split_ascii_whitespace().collect();
}

pub struct Guesser<'a> {
	pub remaining_words: Vec<&'a str>,
	pub guessed: Vec<char>,
	pub word: Vec<Option<char>>,
}

impl<'a> Guesser<'a> {
	pub fn new_from_default(len: usize) -> Self {
		Guesser {
			remaining_words: WORDS.clone(),
			guessed: vec![],
			word: std::iter::repeat(None).take(len).collect(),
		}
	}

	pub fn new_from_dict(dict: &[&'a str], len: usize) -> Self {
		Guesser {
			remaining_words: dict.to_vec(),
			guessed: vec![],
			word: std::iter::repeat(None).take(len).collect(),
		}
	}

	/// Eliminate possibilities which don't match the current truth.
	pub fn elim(&mut self) {
		self.remaining_words.retain(|w| {
			w.len() == self.word.len()
				&& w.chars().enumerate().all(|(n, c)| match self.word.get(n) {
					Some(Some(cw)) if cw == &c => true,
					Some(Some(_)) => false,
					Some(None) => !self.guessed.contains(&c),
					None => false,
				})
		})
	}

	/// Guess a letter
	pub fn guess(&mut self) -> Option<char> {
		let mut occurrences: BTreeMap<char, usize> = BTreeMap::new();
		self.remaining_words.iter().for_each(|word| {
			word.chars()
				.for_each(|char| *occurrences.entry(char).or_insert(0) += 1)
		});
		let mut occurrences: Vec<(char, usize)> = occurrences
			.into_iter()
			.filter(|(c, _)| !self.guessed.contains(c))
			.collect();
		occurrences.sort_by(|(_, a), (_, b)| a.cmp(b));

		let guess = occurrences.last().map(|v| v.0);
		if let Some(c) = &guess {
			self.guessed.push(*c);
		}
		guess
	}

	/// Check if we win
	pub fn win(&self) -> bool {
		self.word.iter().all(|v| v.is_some())
	}

	/// Check if we lost
	pub fn wrong(&self) -> usize {
		let correct: Vec<char> = self.word.iter().flatten().copied().collect();
		self.guessed
			.iter()
			.filter(|c| !correct.contains(*c))
			.count()
	}
}
