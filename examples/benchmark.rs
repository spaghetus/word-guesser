use std::sync::{atomic::Ordering::Relaxed, Arc};

use indicatif::ProgressStyle;
use rayon::prelude::*;
use word_guesser::Guesser;

fn main() {
	let total = word_guesser::WORDS.len();
	let bar = indicatif::ProgressBar::new(total as u64)
		.with_message("Working...")
		.with_style(
			ProgressStyle::with_template(
				"{spinner:.green} {msg} ({pos}/{len}), {percent}% complete with ETA {eta_precise}\r\n> {bar:80.cyan/blue} <",
			)
			.unwrap()
			.progress_chars("█▉▊▋▌▍▎▏  "),
		);
	let wins = Arc::new(std::sync::atomic::AtomicUsize::from(0));
	eprintln!("Starting to guess");
	word_guesser::WORDS
		.par_iter()
		.map(|word| {
			let mut guesser = Guesser::new_from_default(word.len());
			while !guesser.win() {
				guesser.elim();
				if guesser.guess().is_none() {
					return false;
				}
				guesser.word = word
					.chars()
					.map(|c| match guesser.guessed.contains(&c) {
						true => Some(c),
						false => None,
					})
					.collect();
				if guesser
					.guessed
					.iter()
					.filter(|c| !word.contains(&c.to_string()))
					.count() > 5
				{
					return false;
				}
			}
			true
		})
		.inspect({
			let bar = bar.clone();
			let wins = wins.clone();
			move |won| {
				bar.inc(1);
				if *won {
					wins.fetch_add(1, Relaxed);
				}
				bar.set_message(format!(
					"{:.1}% win rate, {:.1}/s",
					100.0 * wins.load(Relaxed) as f64 / bar.position() as f64,
					bar.per_sec()
				))
			}
		})
		.filter(|v| *v)
		.count();
	bar.finish_with_message("Done!");
	let wins = wins.load(Relaxed);
	eprintln!(
		"Guesser won for {} out of {} ({}%) words",
		wins,
		total,
		(wins as f64) * 100.0 / (total as f64)
	)
}
