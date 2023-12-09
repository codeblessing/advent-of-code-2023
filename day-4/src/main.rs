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
}

fn total_cards_score(cards: &str) {
    let total_score = cards.lines().map(card_point_score).sum::<i32>();

    println!("{total_score}");
}

fn card_point_score(card: &str) -> i32 {
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
        let offset = owned.len();
        if offset != 0 {
            return 1 << (offset - 1);
        }
    }

    0
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
