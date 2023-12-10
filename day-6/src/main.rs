#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use itertools::Itertools;
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
    let product = error_margin_product(contents.as_str());
    println!("{product}");
    // Part II
    let product = corrected_error_margin_product(contents.as_str());
    println!("{product}");
}

fn corrected_error_margin_product(contents: &str) -> usize {
    fn parse_input(contents: &str) -> Vec<Race> {
        let (times, distances) = contents
            .lines()
            .collect_tuple()
            .expect("invalid source data");

        let times = [times
            .split_whitespace()
            .skip(1)
            .collect::<String>()
            .as_str()]
        .into_iter()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        .collect_vec();

        let distances = [distances
            .split_whitespace()
            .skip(1)
            .collect::<String>()
            .as_str()]
        .into_iter()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        .collect_vec();

        times
            .into_iter()
            .zip(distances)
            .map(|(time, distance)| Race { time, distance })
            .collect_vec()
    }

    let races = parse_input(contents);

    calculate_distance_count(races.as_slice())
        .into_iter()
        .product()
}

fn error_margin_product(contents: &str) -> usize {
    fn parse_input(contents: &str) -> Vec<Race> {
        let (times, distances) = contents
            .lines()
            .collect_tuple()
            .expect("invalid source data");

        let times = times
            .split_whitespace()
            .skip(1)
            .map(str::parse::<usize>)
            .filter_map(Result::ok)
            .collect_vec();
        let distances = distances
            .split_whitespace()
            .skip(1)
            .map(str::parse::<usize>)
            .filter_map(Result::ok)
            .collect_vec();

        times
            .into_iter()
            .zip(distances)
            .map(|(time, distance)| Race { time, distance })
            .collect_vec()
    }

    let races = parse_input(contents);
    calculate_distance_count(races.as_slice())
        .into_iter()
        .product()
}

fn calculate_distance_count(races: &[Race]) -> Vec<usize> {
    races
        .iter()
        .map(|race| {
            (0..=race.time)
                .map(|time| (race.time - time) * time)
                .filter(|distance| distance > &race.distance)
                .count()
        })
        .collect_vec()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Race {
    time: usize,
    distance: usize,
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

#[cfg(test)]
mod test;
