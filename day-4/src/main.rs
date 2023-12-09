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
    total_cards_score(contents.as_str());

    // Part II
    let cards_count = count_total_cards(contents.as_str());

    println!("{cards_count}");
}

fn total_cards_score(cards: &str) {
    let point_score = |count: usize| if count == 0 { 0 } else { 1 << (count - 1) };
    let total_score = cards
        .lines()
        .map(winning_numbers_count)
        .map(point_score)
        .sum::<i32>();

    println!("{total_score}");
}

fn winning_numbers_count(card: &str) -> usize {
    let offset = card.find(':').unwrap_or(0);
    if let Some((winning, mut owned)) = card[offset..]
        .split('|')
        .map(str::split_whitespace)
        .map(|elements| {
            elements
                .map(str::parse::<i32>)
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
        })
        .collect_tuple()
    {
        owned.retain(|element| winning.contains(element));
        return owned.len();
    }

    0
}

fn create_replication_table(cards: &str) -> Vec<Replication> {
    let mut replication_table = cards
        .lines()
        .map(winning_numbers_count)
        .map(|score| Replication {
            factor: 1,
            record: score,
        })
        .collect::<Vec<_>>();

    for index in 0..replication_table.len() {
        let root = replication_table[index];
        for replication in replication_table
            .iter_mut()
            .skip(index + 1)
            .take(root.record)
        {
            replication.factor += root.factor;
        }
    }

    replication_table
}

fn count_total_cards(cards: &str) -> usize {
    create_replication_table(cards)
        .into_iter()
        .map(|replication| replication.factor)
        .sum::<usize>()
}

#[derive(Clone, Copy, Debug, Default)]
struct Replication {
    factor: usize,
    record: usize,
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
