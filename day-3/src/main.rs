#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use regex::Regex;
use tracing::{error, Level};
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

    let Ok(contents) = std::fs::read_to_string(args.filename.as_ref().unwrap()) else {
        error!("Cannot read file contents.");
        return;
    };

    let line_length = contents.lines().take(1).map(str::len).sum::<usize>() + 1;

    let digit_pattern = Regex::new(r"(\d+)").expect("Invalid pattern");
    let symbol_pattern = Regex::new(r"([^\d\.\n\r])").expect("Invalid symbol pattern");

    let mut numbers = digit_pattern
        .captures_iter(contents.as_str())
        .filter_map(|captures| captures.get(0))
        .map(|element| element.range())
        .collect::<Vec<_>>();
    let symbols = symbol_pattern
        .captures_iter(contents.as_str())
        .filter_map(|captures| captures.get(0))
        .map(|element| element.range())
        .collect::<Vec<_>>();

    numbers.retain(|number| {
            symbols
                .iter()
                .any(|symbol| {
                    // Check previous line for symbol.
                    number.start.saturating_sub(1 + line_length) <= symbol.start && number.end.saturating_sub(line_length - 1) >= symbol.end ||
                    // Check current line for symbol.
                    number.start.saturating_sub(1) <= symbol.start && number.end.saturating_add(1) >= symbol.end ||
                    // Check next line for symbol.
                    number.start.saturating_add(line_length - 1) <= symbol.start && number.end.saturating_add(line_length + 1) >= symbol.end
                })
        });

    // let show_numbers: Vec<_> = numbers
    //     .iter()
    //     .map(|range| &contents[range.clone()])
    //     .collect();
    // let show_symbols: Vec<_> = symbols
    //     .iter()
    //     .map(|range| &contents[range.clone()])
    //     .collect();

    // debug!("{show_numbers:?}");
    // debug!("{show_symbols:?}");

    let sum = numbers
        .into_iter()
        .filter_map(|range| contents[range].parse::<i64>().ok())
        .sum::<i64>();

    println!("{sum}");
    // let mut replaced = contents.to_string();

    // for number in numbers {
    //     replaced.replace_range(
    //         number.clone(),
    //         repeat_n('.', number.len()).collect::<String>().as_str(),
    //     );
    // }

    // print!("{replaced}");
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_name = "<FILE>")]
    filename: Option<PathBuf>,
    #[arg(short, long, value_name = "<COUNT>", action = ArgAction::Set, default_value_t = 0)]
    greens: u32,
    #[arg(short, long, value_name = "<COUNT>", action = ArgAction::Set, default_value_t = 0)]
    blues: u32,
    #[arg(short, long, value_name = "<COUNT>", action = ArgAction::Set, default_value_t = 0)]
    reds: u32,
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}