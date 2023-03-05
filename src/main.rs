extern crate argparse;

use argparse::{ArgumentParser, Store};

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::thread;
use std::time::Duration;

struct ReaderConfig {
    words_per_minute: u64,
    resume_point: u64,
    filename: String,
}

enum WordConcentration {
    SmallWord,
    LessThanFine,
    Fine,
    GreaterThanFine,
    VeryHigh,
}

fn estimate_word_length(word: &str) -> WordConcentration {
    if word
        .chars()
        .any(|s| s == '.' || s == ',' || s == '-' || s == '!' || s == ':' || s.is_uppercase())
    {
        return WordConcentration::VeryHigh;
    }
    match word.len() {
        1..=2 => WordConcentration::SmallWord,
        3..=4 => WordConcentration::LessThanFine,
        5..=12 => WordConcentration::Fine,
        13..=14 => WordConcentration::GreaterThanFine,
        _ => WordConcentration::VeryHigh,
    }
}

fn sleep_time(words_per_minute: u64, word_estimation: WordConcentration) {
    let time_multiplier = match word_estimation {
        WordConcentration::SmallWord => 0.9,
        WordConcentration::LessThanFine => 0.7,
        WordConcentration::Fine => 0.7,
        WordConcentration::GreaterThanFine => 0.7,
        WordConcentration::VeryHigh => 2.4,
    };
    thread::sleep(Duration::from_millis(
        (((1000 * 60 / words_per_minute) as f32) * time_multiplier) as u64,
    ));
}

fn parse_args() -> ReaderConfig {
    let mut resume_point: u64 = 0;
    let mut words_per_minute: u64 = 100;
    let mut filename = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Program that writes out file word by word on console.");

        ap.refer(&mut words_per_minute).add_option(
            &["-w", "--words-count"],
            Store,
            "How many words will be displayed per minute.",
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
        words_per_minute,
        resume_point,
        filename,
    }
}

fn main() {
    let reader_config = parse_args();
    let opened_file = File::open(reader_config.filename).expect("Could not open a file.");
    let file_buffer = BufReader::new(opened_file);

    let mut skipped_words = reader_config.resume_point;

    for line in file_buffer.lines() {
        let line = line.unwrap();
        let words = line.split_whitespace();

        for word in words.into_iter() {
            if skipped_words > 0 {
                skipped_words -= 1;
                continue;
            }
            println!("{}", word);

            sleep_time(reader_config.words_per_minute, estimate_word_length(word));
        }
    }
}
