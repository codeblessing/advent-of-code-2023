#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;
use std::{cmp::Ordering, convert::Infallible};

use clap::{ArgAction, Parser};
use itertools::Itertools;
use parts::Part;
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
    let total = modified_total_winnings(contents.as_str());
    println!("{total}");
}

fn total_winnings(contents: &str) -> usize {
    fn parse_rounds(contents: &str) -> Vec<Round<parts::I>> {
        contents
            .lines()
            .map(Round::from_str)
            .filter_map(Result::ok)
            .sorted_by_cached_key(|round| round.hand.clone())
            .collect()
    }

    let rounds = parse_rounds(contents);

    rounds
        .into_iter()
        .enumerate()
        .map(|(weight, round)| weight.saturating_add(1) * round.bid)
        .sum::<usize>()
}

fn modified_total_winnings(contents: &str) -> usize {
    fn parse_rounds(contents: &str) -> Vec<Round<parts::II>> {
        fn update_hand_type<P: Part>(hand: HandType<P>) -> HandType<P>
        where
            Card<P>: PartialOrd + Ord,
        {
            match hand {
                HandType::HighCard(hand) => {
                    if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::OnePair(hand)
                    } else {
                        HandType::HighCard(hand)
                    }
                }
                HandType::OnePair(hand) => {
                    if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::Three(hand)
                    } else {
                        HandType::OnePair(hand)
                    }
                }
                HandType::TwoPairs(hand) => {
                    if hand.cards.iter().filter(|card| matches!(card, Card::Jack)).count() == 2 {
                        HandType::Four(hand)
                    } else if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::FullHouse(hand)
                    } else {
                        HandType::TwoPairs(hand)
                    }
                }
                HandType::Three(hand) => {
                    if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::Four(hand)
                    } else {
                        HandType::Three(hand)
                    }
                }
                HandType::FullHouse(hand) => {
                    if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::Five(hand)
                    } else {
                        HandType::FullHouse(hand)
                    }
                }
                HandType::Four(hand) => {
                    if hand.cards.iter().any(|card| matches!(card, Card::Jack)) {
                        HandType::Five(hand)
                    } else {
                        HandType::Four(hand)
                    }
                }
                hand @ HandType::Five(_) => hand,
            }
        }

        contents
            .lines()
            .map(Round::<parts::II>::from_str)
            .filter_map(Result::ok)
            .map(|round| Round {
                hand: update_hand_type(round.hand),
                bid: round.bid,
            })
            .sorted_by_cached_key(|round| round.hand.clone())
            .collect_vec()
    }

    let rounds = parse_rounds(contents);

    rounds
        .into_iter()
        .enumerate()
        .map(|(weight, round)| weight.saturating_add(1) * round.bid)
        .sum::<usize>()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Round<P: Part>
where
    Card<P>: PartialOrd + Ord,
{
    hand: HandType<P>,
    bid: usize,
}

impl<P: Part> FromStr for Round<P>
where
    Card<P>: PartialOrd + Ord,
{
    type Err = AoCError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let (cards, bid) = src.split_whitespace().collect_tuple().ok_or(AoCError::RoundParse)?;

        let bid = bid.parse::<usize>().map_err(|_| AoCError::RoundParse)?;

        let hand = cards
            .chars()
            .map(Card::try_from)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect_tuple::<(Card<P>, Card<P>, Card<P>, Card<P>, Card<P>)>()
            .ok_or(AoCError::RoundParse)?;

        let hand = Hand {
            cards: [hand.0, hand.1, hand.2, hand.3, hand.4],
        }
        .into();

        Ok(Round { hand, bid })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType<P: Part>
where
    Card<P>: PartialOrd + Ord,
{
    HighCard(Hand<P>),
    OnePair(Hand<P>),
    TwoPairs(Hand<P>),
    Three(Hand<P>),
    FullHouse(Hand<P>),
    Four(Hand<P>),
    Five(Hand<P>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand<P: Part>
where
    Card<P>: PartialOrd + Ord,
{
    cards: [Card<P>; 5],
}

impl<P: Part> From<Hand<P>> for HandType<P>
where
    Card<P>: PartialOrd + Ord,
{
    fn from(hand: Hand<P>) -> Self {
        let groups = hand
            .cards
            .clone()
            .into_iter()
            .into_group_map_by(std::mem::discriminant)
            .into_values()
            .map(|group| group.len())
            .collect_vec();

        match groups {
            // If we have 5 groups it means we have 5 different cards in hand, hence High Card
            group if group.len() == 5 => HandType::HighCard(hand),
            // If we have 4 groups it means exactly one has 2 cards, hence One Pair
            group if group.len() == 4 => HandType::OnePair(hand),
            // If we have 3 groups and one of them is 3 then others must have 1, hence Three of a Kind
            group if group.len() == 3 && group.iter().any(|&e| e == 3) => HandType::Three(hand),
            // If we have 3 groups and two of them is 2 then other must have 1, hence Two Pairs
            group if group.len() == 3 && group.iter().filter(|&&e| e == 2).count() == 2 => HandType::TwoPairs(hand),
            // If we have 2 groups and one of them is 4 then other is 1, hence Four of a Kind
            group if group.len() == 2 && group.iter().any(|&e| e == 4) => HandType::Four(hand),
            // If we have 2 groups and none of them is 4 then one must be 3 and other 2, hence Full House
            group if group.len() == 2 => HandType::FullHouse(hand),
            // If we have only one group, only option is 5, hence Five of a Kind
            _ => HandType::Five(hand),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Card<P: Part>
where
    Card<P>: PartialOrd + Ord,
{
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
    #[allow(unused)]
    Unreachable(Infallible, PhantomData<P>),
}

impl<P: Part> TryFrom<char> for Card<P>
where
    Card<P>: PartialOrd + Ord,
{
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

impl Ord for Card<parts::I> {
    fn cmp(&self, other: &Self) -> Ordering {
        let me = unsafe { *<*const _>::from(self).cast::<u8>() };
        let other = unsafe { *<*const _>::from(other).cast::<u8>() };

        me.cmp(&other)
    }
}

impl PartialOrd for Card<parts::I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card<parts::II> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Card::Jack => match other {
                Card::Jack => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Two => match other {
                Card::Jack => Ordering::Greater,
                Card::Two => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Three => match other {
                Card::Jack | Card::Two => Ordering::Greater,
                Card::Three => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Four => match other {
                Card::Jack | Card::Two | Card::Three => Ordering::Greater,
                Card::Four => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Five => match other {
                Card::Jack | Card::Two | Card::Three | Card::Four => Ordering::Greater,
                Card::Five => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Six => match other {
                Card::Jack | Card::Two | Card::Three | Card::Four | Card::Five => Ordering::Greater,
                Card::Six => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Seven => match other {
                Card::Jack | Card::Two | Card::Three | Card::Four | Card::Five | Card::Six => Ordering::Greater,
                Card::Seven => Ordering::Equal,
                _ => Ordering::Less,
            },
            Card::Eight => match other {
                Card::Nine | Card::Ten | Card::Queen | Card::King | Card::Ace => Ordering::Less,
                Card::Eight => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::Nine => match other {
                Card::Ten | Card::Queen | Card::King | Card::Ace => Ordering::Less,
                Card::Nine => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::Ten => match other {
                Card::Queen | Card::King | Card::Ace => Ordering::Less,
                Card::Ten => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::Queen => match other {
                Card::King | Card::Ace => Ordering::Less,
                Card::Queen => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::King => match other {
                Card::Ace => Ordering::Less,
                Card::King => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::Ace => match other {
                Card::Ace => Ordering::Equal,
                _ => Ordering::Greater,
            },
            Card::Unreachable(..) => unreachable!(),
        }
    }
}

impl PartialOrd for Card<parts::II> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

mod parts {
    pub trait Part: PartialEq + Eq + PartialOrd + Ord + Clone {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct I;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct II;

    impl Part for I {}
    impl Part for II {}
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
