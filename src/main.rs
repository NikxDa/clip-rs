use anyhow::{bail, Context};
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::VecDeque;

#[derive(Default)]
struct History {
    items: VecDeque<HistoryItem>,
}

struct HistoryItem {
    date: DateTime<Local>,
    contents: String,
}

impl HistoryItem {
    pub fn new<S>(contents: S) -> Self
    where
        S: ToString,
    {
        let contents = contents.to_string();
        Self {
            date: Local::now(),
            contents,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    List {
        amount: Option<usize>,
        #[clap(short, long)]
        all: bool,
    },
    Show {
        pattern: String,
    },
    Copy {
        pattern: String,
    },
    Remove {
        pattern: String,
    },
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // let history = History::default();

    let history = {
        let mut history = History::default();
        history.items.push_back(HistoryItem::new("foo"));
        history.items.push_back(HistoryItem::new("bar"));
        history
    };

    match args.command {
        Commands::List { amount, all } => {
            let amount = if all {
                history.items.len()
            } else {
                amount.unwrap_or(10)
            };
            for (i, item) in history.items.iter().rev().take(amount).rev().enumerate() {
                println!(
                    "{index:03} {content}",
                    index = i.to_string().yellow(),
                    content = item.contents
                );
            }
        }
        Commands::Show { pattern } => {
            let _index = parse_pattern(&history, pattern)?;

            if _index < history.items.len() {
                let item = &history.items[_index];
                println!("{}", item.contents);
            } else {
                println!(
                    "No item with index: {index:03}",
                    index = _index.to_string().yellow()
                );
            }
        }
        Commands::Copy { pattern } => {
            let _index = parse_pattern(&history, pattern)?;
        }
        Commands::Remove { pattern } => {
            let _index = parse_pattern(&history, pattern)?;
        }
    }

    Ok(())
}

fn parse_pattern<S>(history: &History, pattern: S) -> anyhow::Result<usize>
where
    S: AsRef<str>,
{
    let pattern = pattern.as_ref();
    let number_of_items: usize = history.items.len();

    let item_index = match pattern {
        p if p.starts_with('~') => {
            let offset = p
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<usize>()
                .context("Relative offset has to be a number.")?;
            if offset > number_of_items - 1 {
                bail!("Offset out of range")
            }
            number_of_items.saturating_sub(1 + offset)
        }
        p => p.parse().context("History item has to be a number")?,
    };

    if item_index > number_of_items - 1 {
        bail!("Index out of range")
    }

    Ok(item_index)
}

#[cfg(test)]
mod tests {
    use crate::{parse_pattern, History, HistoryItem};

    #[test]
    fn test_parse_pattern() -> anyhow::Result<()> {
        let history = {
            let mut history = History::default();
            history.items.push_back(HistoryItem::new("foo"));
            history.items.push_back(HistoryItem::new("bar"));
            history
        };

        assert_eq!(parse_pattern(&history, "0")?, 0);
        assert_eq!(parse_pattern(&history, "~0")?, 1);
        assert_eq!(parse_pattern(&history, "~1")?, 0);

        Ok(())
    }
}
