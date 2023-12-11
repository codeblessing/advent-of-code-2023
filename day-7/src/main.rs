#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::{path::PathBuf, str::FromStr};

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
    let total = total_winnings(contents.as_str());
    println!("{total}");
    // Part II
}

fn total_winnings(contents: &str) -> usize {
    let rounds = parse_rounds(contents);

    rounds
        .into_iter()
        .enumerate()
        .map(|(weight, round)| weight.saturating_add(1) * round.bid)
        .sum::<usize>()
}

fn parse_rounds(contents: &str) -> Vec<Round> {
    contents
        .lines()
        .map(Round::from_str)
        .filter_map(Result::ok)
        .sorted_by_key(|round| round.hand.clone())
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Round {
    hand: HandType,
    bid: usize,
}

impl FromStr for Round {
    type Err = AoCError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let Some((cards, bid)) = src.split_whitespace().collect_tuple() else {
            return Err(AoCError::RoundParse);
        };

        let bid = bid.parse::<usize>().map_err(|_| AoCError::RoundParse)?;

        let hand = cards
            .chars()
            .map(Card::try_from)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect_tuple::<(Card, Card, Card, Card, Card)>()
            .ok_or(AoCError::RoundParse)?;

        let hand = Hand(hand.0, hand.1, hand.2, hand.3, hand.4).into();

        Ok(Round { hand, bid })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard(Hand),
    OnePair(Hand),
    TwoPairs(Hand),
    Three(Hand),
    FullHouse(Hand),
    Four(Hand),
    Five(Hand),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand(Card, Card, Card, Card, Card);

impl From<Hand> for HandType {
    fn from(val: Hand) -> Self {
        let groups = [val.0, val.1, val.2, val.3, val.4]
            .into_iter()
            .into_group_map_by(|&card| card as u8)
            .into_values()
            .map(|group| group.len())
            .collect_vec();

        match groups {
            // If we have 5 groups it means we have 5 different cards in hand, hence High Card
            group if group.len() == 5 => HandType::HighCard(val),
            // If we have 4 groups it means exactly one has 2 cards, hence One Pair
            group if group.len() == 4 => HandType::OnePair(val),
            // If we have 3 groups and one of them is 3 then others must have 1, hence Three of a Kind
            group if group.len() == 3 && group.iter().any(|&e| e == 3) => HandType::Three(val),
            // If we have 3 groups and two of them is 2 then other must have 1, hence Two Pairs
            group if group.len() == 3 && group.iter().filter(|&&e| e == 2).count() == 2 => HandType::TwoPairs(val),
            // If we have 2 groups and one of them is 4 then other is 1, hence Four of a Kind
            group if group.len() == 2 && group.iter().any(|&e| e == 4) => HandType::Four(val),
            // If we have 2 groups and none of them is 4 then one must be 3 and other 2, hence Full House
            group if group.len() == 2 => HandType::FullHouse(val),
            // If we have only one group, only option is 5, hence Five of a Kind
            _ => HandType::Five(val),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = AoCError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::Ace),
            'K' => Ok(Self::King),
            'Q' => Ok(Self::Queen),
            'J' => Ok(Self::Jack),
            'T' => Ok(Self::Ten),
            '9' => Ok(Self::Nine),
            '8' => Ok(Self::Eight),
            '7' => Ok(Self::Seven),
            '6' => Ok(Self::Six),
            '5' => Ok(Self::Five),
            '4' => Ok(Self::Four),
            '3' => Ok(Self::Three),
            '2' => Ok(Self::Two),
            card => Err(AoCError::CardParse(card)),
        }
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

#[derive(Clone, Debug, Error)]
enum AoCError {
    #[error("cannot parse round record")]
    RoundParse,
    #[error("cannot parse card. Unknown card `{0}`")]
    CardParse(char),
}

#[cfg(test)]
mod test;
