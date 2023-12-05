#![warn(clippy::pedantic)]
#![warn(clippy::all)]

use std::path::PathBuf;

use clap::{ArgAction, Parser};
use tracing::{debug, error, info, trace, Level};
use tracing_subscriber::FmtSubscriber;

fn main() {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("[ERROR] Cannot set up logging.");
        eprintln!("Error cause: {error}");
    };

    if args.filename.is_none() {
        error!("Inputs other than file are currently not supported.");
        return;
    }

    let Ok(contents) = std::fs::read_to_string(args.filename.unwrap()) else {
        error!("Cannot read file contents.");
        return;
    };

    let replacements = vec![
        ("one", "1"),
        ("two", "2"),
        ("three", "3"),
        ("four", "4"),
        ("five", "5"),
        ("six", "6"),
        ("seven", "7"),
        ("eight", "8"),
        ("nine", "9"),
    ];

    let sum = contents
        .lines()
        .map(|line| {
            debug!("Original string: {line}");
            let mut line = line.to_string();
            // This is one way to do it
            let mut first = replacements
                .iter()
                .filter_map(|(value, replacement)| {
                    if let Some(offset) = line.find(value) {
                        Some((offset, value, replacement))
                    } else {
                        None
                    }
                })
                .min_by_key(|&(offset, ..)| offset)
                .map(|(offset, value, replacement)| {
                    // Unfortunately this has to be done this way, because digit-words can overlap.
                    ((offset + 1)..(offset + value.len() - 1), replacement)
                });

            let last = replacements
                .iter()
                .filter_map(|(value, replacement)| {
                    if let Some(offset) = line.rfind(value) {
                        Some((offset, value, replacement))
                    } else {
                        None
                    }
                })
                .max_by_key(|&(offset, ..)| offset)
                .map(|(offset, value, replacement)| {
                    // See comment up.
                    ((offset + 1)..(offset + value.len() - 1), replacement)
                });

            // Special case: replacements overlap. If that's the case we replace only first element.
            if let (Some((first_range, ..)), Some((last_range, ..))) = (&first, &last) {
                if first_range.end > last_range.start {
                    first = None;
                }
            }

            // We replace last occurence at first, because it will not change offsets.
            // If first occurence was replaced at first, this could change offsets and invoke panic.
            if let Some((range, replacement)) = last {
                line.replace_range(range, &replacement);
            }

            if let Some((range, replacement)) = first {
                line.replace_range(range, &replacement);
            }

            // Alternatively define replacements as below and do full replacement:
            // let replacements = vec![
            //     ("one", "o1e"),
            //     ("two", "t2o"),
            //     ("three", "t3e"),
            //     ("four", "f4r"),
            //     ("five", "f5e"),
            //     ("six", "s6x"),
            //     ("seven", "s7n"),
            //     ("eight", "e8t"),
            //     ("nine", "n9e"),
            // ];
            // replacements.iter().for_each(|(value, replacement)| line = line.replace(value, &replacement));
            debug!("Replaced string: {line}");
            line
        })
        .map(|input| {
            input
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<Vec<_>>()
        })
        .filter_map(|digits| {
            trace!("Numbers in line: {digits:?}");
            match (digits.first().copied(), digits.last().copied()) {
                (Some(first), Some(last)) => Some(format!("{first}{last}")),
                _ => None,
            }
        })
        .filter_map(|value| {
            info!("Line value: {value}");
            value.parse::<u32>().ok()
        })
        .reduce(|u, v| u + v)
        .unwrap_or(0);

    println!("{sum}");
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_name = "<FILE>")]
    filename: Option<PathBuf>,
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}
