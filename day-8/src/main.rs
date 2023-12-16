#![warn(clippy::all)]
#![warn(clippy::pedantic)]
use std::{collections::HashMap, path::PathBuf, str::FromStr};

use clap::{ArgAction, Parser};
use itertools::Itertools;
use num_integer::lcm as lowest_common_multiple;
use rayon::prelude::*;
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
    let steps = step_count(contents.as_str());
    println!("{steps}");
    // Part II
    let steps = simultaneous_step_count(contents.as_str());
    println!("{steps}");
}

fn step_count(contents: &str) -> usize {
    let map = Map::from_str(contents).expect("Invalid map");

    let mut moves = map.moves.into_iter();

    let mut steps = 0;
    let mut node = map
        .nodes
        .get(&Id("AAA".to_owned()))
        .expect("Map has no starting point");
    loop {
        steps += 1;
        match moves.next().unwrap() {
            Direction::Left => node = map.nodes.get(&node.left).expect("Node doesn't exist."),
            Direction::Right => node = map.nodes.get(&node.right).expect("Node doesn't exist."),
        }

        if node.id.0.as_str() == "ZZZ" {
            break;
        }
    }

    steps
}

fn simultaneous_step_count(contents: &str) -> usize {
    // Finding common end path could take forever going with naive solution.
    // Fortunately, we can calculate indices of ending points for every path
    // and then just find lowest common multiple of them!

    let map = Map::from_str(contents).expect("Invalid map");

    let moves = map.moves.into_iter();
    let nodes = map
        .nodes
        .values()
        .par_bridge()
        .filter(|node| node.is_origin())
        .map(|node| {
            let mut path = Vec::new();
            let moves = moves.clone();

            let mut node = node;
            let len = moves.scheme.len();

            for (index, direction) in moves.enumerate() {
                match direction {
                    Direction::Left => {
                        node = map.nodes.get(&node.left).expect("Node doesn't exist.");
                        if node.is_final() {
                            path.push((index + 1, node.clone()));
                        }
                    }
                    Direction::Right => {
                        node = map.nodes.get(&node.right).expect("Node doesn't exist.");
                        if node.is_final() {
                            path.push((index + 1, node.clone()));
                        }
                    }
                }

                if !path.is_empty() || index > (len * len) {
                    break;
                }
            }

            let last = path.last().cloned().unwrap();
            path.push(last);

            path
        })
        .collect::<Vec<_>>();

    let steps = nodes
        .iter()
        .map(|path| path.iter().map(|&(index, _)| index))
        .multi_cartesian_product()
        .map(|indices| {
            indices
                .into_iter()
                .reduce(lowest_common_multiple)
                .unwrap_or_default()
        })
        .min()
        .unwrap_or_default();

    steps
}

#[derive(Clone, Debug, PartialEq)]
struct Map {
    moves: Moves,
    nodes: HashMap<Id, Node>,
}

impl FromStr for Map {
    type Err = AoCError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let (moves, nodes) = src.split_once("\n\n").ok_or(AoCError::InvalidSource)?;

        let moves = moves
            .chars()
            .map(|direction| match direction {
                'L' => Ok(Direction::Left),
                'R' => Ok(Direction::Right),
                unknown => Err(AoCError::InvalidDirection(unknown)),
            })
            .collect::<Result<Vec<Direction>, AoCError>>()?;

        let moves = Moves { scheme: moves };

        let nodes = nodes
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| line.split_once(" = "))
            .map(|(id, neighbors)| {
                let id = Id::from_str(id).unwrap();
                let (left, right) = neighbors
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .split(',')
                    .map(str::trim)
                    .map(Id::from_str)
                    .map(Result::unwrap)
                    .collect_tuple::<(Id, Id)>()
                    .ok_or(AoCError::InvalidNode)?;

                Ok::<_, AoCError>((id.clone(), Node { id, left, right }))
            })
            .collect::<Result<HashMap<Id, Node>, AoCError>>()?;

        Ok(Self { moves, nodes })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Node {
    id: Id,
    left: Id,
    right: Id,
}

impl Node {
    #[allow(unused)]
    fn is_origin(&self) -> bool {
        self.id.0.ends_with('A')
    }

    fn is_final(&self) -> bool {
        self.id.0.ends_with('Z')
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Id(String);

impl FromStr for Id {
    type Err = AoCError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(Self(src.to_owned()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Moves {
    scheme: Vec<Direction>,
}

#[derive(Clone, Debug, PartialEq)]
struct MoveIterator {
    index: usize,
    scheme: Vec<Direction>,
}

impl Iterator for MoveIterator {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.scheme.len();
        let index = self.index;
        self.index = (index + 1) % len;
        self.scheme.get(index).copied()
    }
}

impl IntoIterator for Moves {
    type Item = Direction;

    type IntoIter = MoveIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index: 0,
            scheme: self.scheme,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
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
enum AoCError {
    #[error("source file has invalid format")]
    InvalidSource,
    #[error("direction can be only `L` or `R`. `{0}` is neither of them.")]
    InvalidDirection(char),
    #[error("node entry has invalid format")]
    InvalidNode,
}

#[cfg(test)]
mod test;
