#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use itertools::Itertools;
use thiserror::Error;
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

    // Part I
    let sum = sum_of_successors(contents.as_str());
    println!("{sum}");
    // Part II
}

fn sum_of_successors(contents: &str) -> i64 {
    let lists = parse_list(contents);

    lists
        .into_iter()
        .map(|list| {
            let mut lasts = vec![list.last().copied().unwrap()];
            let mut current = generate_diff_list(&list);

            // We generate difference lists until there's no difference between elements.
            while !current.iter().all(|num| num == &0) {
                lasts.push(current.last().copied().unwrap());
                current = generate_diff_list(current.as_slice());
            }

            lasts.into_iter().sum::<i64>()
        })
        .sum::<i64>()
}

fn parse_list(contents: &str) -> Vec<Vec<i64>> {
    contents
        .lines()
        .map(|line| {
            line.split_whitespace()
                .filter_map(|number| number.parse::<i64>().ok())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn generate_diff_list<'a>(list: &[i64]) -> Vec<i64> {
    list.windows(2).map(|pair| pair[1] - pair[0]).collect_vec()
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_name = "<FILE>")]
    filename: Option<PathBuf>,
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Error)]
enum AoCError {}

#[cfg(test)]
mod test;
