use eyre::eyre;
use std::collections::HashSet;
use std::fs;

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut sum = 0u32;
    let mut card_wins = Vec::new();
    for (lineno, line) in body.lines().enumerate() {
        let (_, num_str) =
            line.split_once(':').ok_or_else(|| eyre!("{}:{}: invalid line", fname, lineno + 1))?;
        let (winner_str, scratch_str) = num_str
            .split_once('|')
            .ok_or_else(|| eyre!("{}:{}: invalid line", fname, lineno + 1))?;
        let mut winners = HashSet::<u32>::new();
        for token in winner_str.split_whitespace() {
            winners.insert(token.parse().map_err(|e| {
                eyre!("{}:{}: {} parsing winner '{}'", fname, lineno + 1, e, token)
            })?);
        }
        let mut wins = 0u32;
        for token in scratch_str.split_whitespace() {
            let num = token.parse::<u32>().map_err(|e| {
                eyre!("{}:{}: {} parsing scratch '{}'", fname, lineno + 1, e, token)
            })?;
            if winners.contains(&num) {
                wins += 1;
            }
        }
        card_wins.push(wins);
        if wins > 0 {
            sum += 1 << (wins - 1);
        }
    }
    println!("{}", sum);
    let mut card_count: Vec<_> = std::iter::repeat(1u32).take(card_wins.len()).collect();
    for (i, wins) in card_wins.into_iter().enumerate() {
        let current = card_count[i];
        for x in 0..wins {
            card_count[i + x as usize + 1] += current;
        }
    }
    println!("{}", card_count.into_iter().sum::<u32>());
    Ok(())
}
