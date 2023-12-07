use std::{cmp::Reverse, collections::BTreeMap};
use std::fs;
use std::collections::HashMap;
use std::str::FromStr;
use eyre::{eyre, Report, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    Ace
}

impl FromStr for Card {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2" => Ok(Card::Two),
            "3" => Ok(Card::Three),
            "4" => Ok(Card::Four),
            "5" => Ok(Card::Five),
            "6" => Ok(Card::Six),
            "7" => Ok(Card::Seven),
            "8" => Ok(Card::Eight),
            "9" => Ok(Card::Nine),
            "T" => Ok(Card::Ten),
            "J" => Ok(Card::Jack),
            "Q" => Ok(Card::Queen),
            "K" => Ok(Card::King),
            "A" => Ok(Card::Ace),
            _ => Err(eyre!("unknown card '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandKind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Hand<T> {
    kind: HandKind,
    cards: Vec<T>
}

impl FromStr for Hand<Card> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 5 {
            return Err(eyre!("hand length {} must be 5", s.len()));
        }
        let cards = s.chars().map(|c| c.to_string().parse::<Card>()).collect::<Result<Vec<_>>>()?;
        let mut counts = HashMap::new();
        for card in cards.iter() {
            *counts.entry(*card).or_insert(0) += 1;
        }
        let mut counts: Vec<_> = counts.into_values().collect();
        counts.sort_by_key(|c| Reverse(*c));
        let kind = match counts[0] {
            5 => HandKind::FiveOfAKind,
            4 => HandKind::FourOfAKind,
            3 => match counts[1] {
                2 => HandKind::FullHouse,
                _ => HandKind::ThreeOfAKind,
            },
            2 => match counts[1] {
                2 => HandKind::TwoPair,
                _ => HandKind::OnePair
            },
            1 => HandKind::HighCard,
            _ => panic!("logic error")
        };
        Ok(Hand { kind, cards} )
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum JokerCard {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Queen,
    King,
    Ace
}

impl FromStr for JokerCard {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2" => Ok(JokerCard::Two),
            "3" => Ok(JokerCard::Three),
            "4" => Ok(JokerCard::Four),
            "5" => Ok(JokerCard::Five),
            "6" => Ok(JokerCard::Six),
            "7" => Ok(JokerCard::Seven),
            "8" => Ok(JokerCard::Eight),
            "9" => Ok(JokerCard::Nine),
            "T" => Ok(JokerCard::Ten),
            "J" => Ok(JokerCard::Joker),
            "Q" => Ok(JokerCard::Queen),
            "K" => Ok(JokerCard::King),
            "A" => Ok(JokerCard::Ace),
            _ => Err(eyre!("unknown card '{}'", s)),
        }
    }
}

impl FromStr for Hand<JokerCard> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 5 {
            return Err(eyre!("hand length {} must be 5", s.len()));
        }
        let cards = s.chars().map(|c| c.to_string().parse::<JokerCard>()).collect::<Result<Vec<_>>>()?;
        let mut counts = HashMap::new();
        let mut jokers = 0;
        for card in cards.iter() {
            match card {
                JokerCard::Joker => jokers += 1,
                _ => *counts.entry(*card).or_insert(0) += 1,
            };
        }
        let mut counts: Vec<_> = counts.into_values().collect();
        counts.sort_by_key(|c| Reverse(*c));
        let kind = match jokers {
            5|4 => HandKind::FiveOfAKind,
            3 => match counts[0] {
                2 => HandKind::FiveOfAKind,
                _ => HandKind::FourOfAKind,
            },
            2 => match counts[0] {
                3 => HandKind::FiveOfAKind,
                2 => HandKind::FourOfAKind,
                _ => HandKind::ThreeOfAKind
            },
            1 => match counts[0] {
                4 => HandKind::FiveOfAKind,
                3 => HandKind::FourOfAKind,
                2 => match counts[1] {
                    2 => HandKind::FullHouse,
                    _ => HandKind::ThreeOfAKind
                },
                1 => HandKind::OnePair,
                _ => panic!("logic error")
            },
            0 => match counts[0] {
                5 => HandKind::FiveOfAKind,
                4 => HandKind::FourOfAKind,
                3 => match counts[1] {
                    2 => HandKind::FullHouse,
                    _ => HandKind::ThreeOfAKind,
                },
                2 => match counts[1] {
                    2 => HandKind::TwoPair,
                    _ => HandKind::OnePair
                },
                1 => HandKind::HighCard,
                _ => panic!("logic error")
            },
            _ => panic!("logic error")
        };
        Ok(Hand { kind, cards} )
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let lines = body.lines();
    let mut rankings = BTreeMap::new();
    let mut joker_rankings = BTreeMap::new();
    for (lineno, line) in lines.enumerate() {
        let (hand_str, ranking_str) = line.split_once(' ').ok_or(eyre!("{}:{}: no split", fname, lineno+1))?;
        let hand = hand_str.parse::<Hand<Card>>()?;
        let joker_hand = hand_str.parse::<Hand<JokerCard>>()?;
        let ranking: u32 = ranking_str.parse()?;
        rankings.insert(hand, ranking);
        joker_rankings.insert(joker_hand, ranking);
    }
    let winnings: usize = rankings.into_iter().enumerate().map(|(i, (_, bet))| (i + 1) * bet as usize).sum();
    let joker_winnings: usize = joker_rankings.into_iter().enumerate().map(|(i, (_, bet))| (i + 1) * bet as usize).sum();
    println!("{}", winnings);
    println!("{}", joker_winnings);
    Ok(())
}
