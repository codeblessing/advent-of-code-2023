#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::{ops::Deref, path::PathBuf, process::exit};

use clap::{ArgAction, Parser};
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

    let mut games = Vec::with_capacity(10);

    for line in contents.lines() {
        let Some(game) = Game::parse(line) else {
            error!("Cannot parse game record.");
            continue;
        };
        games.push(game);
    }

    let sum = sum_of_possible_games(&games, &args);
    let power = power_of_games(&games);

    println!("{sum}");
    println!("{power}");
}

fn power_of_games(games: &[Game]) -> u32 {
    games
        .iter()
        .map(|game| *game.red * *game.green * *game.blue)
        .sum()
}

fn sum_of_possible_games(games: &[Game], args: &Args) -> u32 {
    return games
        .iter()
        .filter(|game| {
            *game.red <= args.reds && *game.green <= args.greens && *game.blue <= args.blues
        })
        .map(|game| game.id)
        .reduce(|u, v| u + v)
        .unwrap_or(0);
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Game {
    id: u32,
    red: Red,
    green: Green,
    blue: Blue,
}

impl Game {
    fn parse(record: &str) -> Option<Self> {
        let data: Vec<_> = record.split(':').collect();
        if data.len() != 2 {
            return None;
        }

        // We checked that record is correct and we have exactly two parts, so we can unwrap them.

        let Some(id) = data.first() else {
            unreachable!()
        };
        let Some(rounds) = data.last() else {
            unreachable!()
        };

        // ID part should follow given schema 'Game <uint id>' but we need only '<numeric id>' part.
        let Ok(id) = id.trim().trim_start_matches("Game ").trim().parse::<u32>() else {
            error!("Invalid Game ID: {id}.");
            exit(0);
        };

        let rounds: Vec<_> = rounds.split(';').map(Round::parse).collect();

        Some(Game {
            id,
            red: rounds
                .iter()
                .map(|round| round.red)
                .max()
                .unwrap_or_default(),
            green: rounds
                .iter()
                .map(|round| round.green)
                .max()
                .unwrap_or_default(),
            blue: rounds
                .iter()
                .map(|rounds| rounds.blue)
                .max()
                .unwrap_or_default(),
        })
    }
}

struct Round {
    red: Red,
    green: Green,
    blue: Blue,
}

impl Round {
    fn parse(round: &str) -> Round {
        let mut red = Red::default();
        let mut green = Green::default();
        let mut blue = Blue::default();

        round
            .split(',')
            .map(str::trim)
            .map(str::split_whitespace)
            .map(Iterator::collect::<Vec<_>>)
            .map(|parts| {
                if parts.len() == 2 {
                    let Ok(count) = parts.first().unwrap().parse::<u32>() else {
                        return Dice::Empty;
                    };

                    match *parts.last().unwrap() {
                        "red" => Dice::Red(count),
                        "green" => Dice::Green(count),
                        "blue" => Dice::Blue(count),
                        _ => Dice::Empty,
                    }
                } else {
                    Dice::Empty
                }
            })
            .for_each(|dice| match dice {
                Dice::Empty => {}
                Dice::Red(counter) => red = Red(counter),
                Dice::Green(counter) => green = Green(counter),
                Dice::Blue(counter) => blue = Blue(counter),
            });

        Self { red, green, blue }
    }
}

enum Dice {
    Empty,
    Red(u32),
    Green(u32),
    Blue(u32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Red(u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Green(u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Blue(u32);

impl Deref for Red {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Green {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Blue {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
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
