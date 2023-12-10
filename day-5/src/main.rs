#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::collections::HashMap;
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

    let data = contents.split("\n\n").collect::<Vec<_>>();
    let maps = &data[1..];
    let mapping = seed_to_location(maps);

    // Part I
    let seeds = data[0]
        .split(':')
        .skip(1)
        .map(str::trim)
        .collect::<String>()
        .split_whitespace()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let locations = seeds.into_iter().map(mapping).collect::<Vec<_>>();

    let location = locations.into_iter().min().unwrap();

    println!("{location}");
    // Part II

    let seeds = data[0]
        .split(':')
        .skip(1)
        .map(str::trim)
        .collect::<String>()
        .split_whitespace()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        .chunks(2)
        .into_iter()
        .map(Iterator::collect::<Vec<usize>>)
        .filter(|chunk| chunk.len() == 2)
        .filter_map(|chunk| {
            chunk
                .into_iter()
                .collect_tuple()
                .map(|(start, len)| start..(start + len))
        })
        .collect::<Vec<_>>();

    let mapping = seed_to_location(maps);

    let location = seeds.into_iter().flat_map(IntoIterator::into_iter).map(mapping).min().unwrap();

    println!("{location}");
}

fn seed_to_location(maps: &[&str]) -> impl Fn(usize) -> usize {
    let maps = maps
        .iter()
        .map(|map| map.split(':').map(str::trim).collect_tuple())
        .map(Option::unwrap)
        .map(|(name, ranges)| {
            (
                name,
                ranges
                    .lines()
                    .map(str::split_whitespace)
                    .map(|nums| {
                        nums.map(str::parse::<usize>)
                            .filter_map(Result::ok)
                            .collect_tuple::<(usize, usize, usize)>()
                            .unwrap()
                    })
                    .map(|(dst, src, len)| ((src..src + len), (dst..dst + len)))
                    .collect::<Vec<_>>(),
            )
        })
        .map(|(name, mappings)| (name.trim_end_matches(" map").to_owned(), mappings))
        .collect::<HashMap<_, _>>();

    move |input: usize| {
        let map_names = [
            "seed-to-soil",
            "soil-to-fertilizer",
            "fertilizer-to-water",
            "water-to-light",
            "light-to-temperature",
            "temperature-to-humidity",
            "humidity-to-location",
        ];

        let mut value = input;
        for map in map_names {
            value = maps[map]
                .iter()
                .filter(|(src, _)| src.contains(&value))
                .map(|(src, dst)| {
                    let offset = value - src.start;
                    dst.start + offset
                })
                .next()
                .unwrap_or(value);
        }

        value
    }
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
