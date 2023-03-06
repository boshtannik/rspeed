extern crate argparse;

use argparse::{ArgumentParser, Store};

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ctrlc;

struct ReaderConfig {
    average_word_length: f32,
    words_per_minute: u64,
    resume_point: u64,
    filename: String,
}

/// This function says - if the word has punctuation in it.
/// Which means, that this word needs bit more
/// time to understand it.
fn is_intense_word(word: &str) -> bool {
    let punctuations = ".,-!?()";

    for letter in punctuations.chars() {
        if word.contains(letter) {
            return true;
        }
    }

    false
}

fn sleep_time(config: &ReaderConfig, word: &str) {
    let ms_per_word: u64 = 1000 * 60 / config.words_per_minute;
    let ms_per_letter: u64 = ms_per_word / config.average_word_length as u64;

    let sleep_duration = (ms_per_letter) * (word.chars().collect::<Vec<char>>().len() as u64)
        + if is_intense_word(word) { 4 } else { 0 };

    thread::sleep(Duration::from_millis(sleep_duration));
}

fn parse_args() -> ReaderConfig {
    let mut resume_point: u64 = 0;
    let mut words_per_minute: u64 = 100;
    let mut filename = String::new();
    let mut average_word_length: f32 = 4.7; // Size of average word in english language.

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Program that writes out file word by word on console.");

        ap.refer(&mut words_per_minute).add_option(
            &["-w", "--words-count"],
            Store,
            "How many words will be displayed per minute.",
        );

        ap.refer(&mut average_word_length).add_option(
            &["-l", "--word-length"],
            Store,
            "Size of average word in the language",
        );

        ap.refer(&mut resume_point).add_option(
            &["-r", "--resume"],
            Store,
            "Number of word, which start from (Resume point)",
        );

        ap.refer(&mut filename)
            .add_option(
                &["-f", "--file-name"],
                Store,
                "Path to file, which will be opened for reading",
            )
            .required();

        ap.parse_args_or_exit();
    }

    ReaderConfig {
        average_word_length,
        words_per_minute,
        resume_point,
        filename,
    }
}

fn main() {
    let words_red_origin = Arc::new(Mutex::new(0u128));

    let words_red_read = Arc::clone(&words_red_origin);
    ctrlc::set_handler(move || {
        let locked_val = words_red_read.lock().unwrap();
        println!("\n\nCtrl + C Event caught, quitting.\nTo start from exact this point - start this program with\nflag: -r {}", locked_val);
        exit(0);
    })
    .expect("Could not set handler to handle Ctrl+C event");

    let reader_config = parse_args();
    let opened_file = File::open(reader_config.filename.clone()).expect("Could not open a file.");
    let file_buffer = BufReader::new(opened_file);

    let mut skipped_words = reader_config.resume_point;

    let words_red_changing = Arc::clone(&words_red_origin);

    for line in file_buffer.lines() {
        let line = line.unwrap();
        let words = line.split_whitespace();

        for word in words.into_iter() {
            if skipped_words > 0 {
                skipped_words -= 1;
                continue;
            }
            println!("{}", word);

            *words_red_changing.lock().unwrap() += 1;

            sleep_time(&reader_config, word);
        }
    }
}
