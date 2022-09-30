use std::{
	sync::{Arc, RwLock},
	thread::JoinHandle,
};

use eframe::{App, NativeOptions};
use word_guesser::Guesser;

fn main() {
	eframe::run_native(
		"Word Guesser",
		NativeOptions::default(),
		Box::new(|_| Box::new(State::default())),
	)
}

#[derive(Default)]
struct State {
	pub guesser: Option<Arc<RwLock<Guesser<'static>>>>,
	pub guesser_thread: Option<JoinHandle<Option<char>>>,
	pub current_guess: Option<Option<char>>,
	pub word_len: usize,
}

impl App for State {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
		eframe::egui::CentralPanel::default().show(ctx, |ui| match &self.guesser {
			None => {
				ui.label("To start the game, set the length of your word.");
				ui.horizontal(|ui| {
					if ui.button("<").clicked() && self.word_len > 0 {
						self.word_len -= 1;
					}
					ui.label(self.word_len.to_string());
					if ui.button(">").clicked() && self.word_len < usize::MAX {
						self.word_len += 1;
					}
				});
				if ui.button("Go!").clicked() {
					let len = self.word_len;
					let g = Arc::new(RwLock::new(Guesser::new_from_default(len)));
					self.guesser = Some(g.clone());
				}
			}
			Some(g) => {
				match (self.guesser_thread.as_ref(), self.current_guess.as_ref()) {
					(None, None) => {
						// Run a guess.
						let g = self.guesser.as_ref().unwrap().clone();
						self.guesser_thread = Some(std::thread::spawn(move || {
							g.as_ref().write().unwrap().elim();
							g.as_ref().write().unwrap().guess()
						}));
						ui.label("Starting...");
						ctx.request_repaint();
					}
					(Some(_), None) => {
						if self.guesser_thread.as_ref().unwrap().is_finished() {
							// The guess is finished!
							ui.label("Done!");
							self.current_guess =
								Some(self.guesser_thread.take().unwrap().join().unwrap_or(None));
							ctx.request_repaint();
						} else {
							ui.label("Working...");
						}
					}
					(None, _) if g.read().unwrap().wrong() > 5 => {
						ui.label("The computer ran out of guesses!");
						if ui.button("Restart").clicked() {
							self.guesser = None;
							self.current_guess = None;
							self.guesser_thread = None;
							self.word_len = 0;
						}
					}
					(None, Some(Some(guess))) => {
						ui.label(format!("The computer guesses {}!", guess));
						ui.label(format!(
							"The computer has {} guesses left, and is considering {}.",
							5 - g.read().unwrap().wrong(),
							match g.read().unwrap().remaining_words.len() {
								n if n < 5 => {
									g.read().unwrap().remaining_words.join(", ")
								}
								n => format!("{} words", n),
							}
						));
						ui.label("Click on the letters it got correct!");
						ui.horizontal_wrapped(|ui| {
							for char in g.write().unwrap().word.iter_mut() {
								match *char {
									Some(c) if c == *guess => {
										if ui.button(c.to_string()).clicked() {
											*char = None;
										}
									}
									Some(c) => {
										ui.label(c.to_string());
									}
									None => {
										if ui.button("_").clicked() {
											*char = Some(*guess)
										}
									}
								}
							}
						});
						if ui.button("Next guess").clicked() {
							self.current_guess = None;
						}
					}
					(None, Some(None)) => {
						if g.read().unwrap().win() {
							ui.label(format!(
								"The computer wins! Your word was {}.",
								g.read().unwrap().word.iter().flatten().collect::<String>(),
							));
						} else {
							ui.label("You win!!!");
						}
						if ui.button("Restart").clicked() {
							self.guesser = None;
							self.current_guess = None;
							self.guesser_thread = None;
							self.word_len = 0;
						}
					}
					(Some(_), Some(_)) => unreachable!(),
				}
			}
		});
	}
}
