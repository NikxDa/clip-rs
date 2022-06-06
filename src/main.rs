use anyhow::{bail, Context};
use arboard::Clipboard;
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

    let mut clipboard = Clipboard::new().unwrap();

    match args.command {
        Commands::List { amount, all } => {
            let amount = amount.unwrap_or(if all { history.items.len() } else { 10 });

            // let max_digits = (history.items.len() + 1).log10() + 1;
            // TOOD: Use this to format index instead of :03

            for (i, item) in history.items.iter().rev().take(amount).rev().enumerate() {
                println!(
                    "{index:03} {content}",
                    index = i.to_string().yellow(),
                    content = item.contents
                );
            }
        }
        Commands::Show { pattern } => {
            let index = parse_pattern(&history, pattern)?;
            let item = &history.items[index];
            println!("{}", item.contents);
        }
        Commands::Copy { pattern } => {
            let index = parse_pattern(&history, pattern)?;
            let item = &history.items[index];
            clipboard.set_text(item.contents.clone())?;

            println!(
                "Successfully copied item {index} to the clipboard!",
                index = index.to_string().yellow()
            )
        }
        Commands::Remove { pattern } => {
            let _index = parse_pattern(&history, pattern)?;
        }
    }

    Ok(())
}

/// Parse an offset pattern to find a specific history item index.
///
/// Offset patterns are either a number or the character `~` followed by a number or nothing.
///
/// # EBNF
/// ```ebnf
/// Digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
/// Number = { Digit };
/// Pattern = Number | '~', [ Number ];
/// ```
///
/// # Behavior
/// If the pattern is a number, it is interpreted as the index of the item to show.<br>
/// If the pattern is `~`, it is interpreted as the index of the last item.<br>
/// If the pattern is `~` followed by a number, it is interpreted as `len - number`.
fn parse_pattern<S>(history: &History, pattern: S) -> anyhow::Result<usize>
where
    S: AsRef<str>,
{
    let pattern = pattern.as_ref();
    let number_of_items: usize = history.items.len();

    let item_index = match pattern {
        p if p.starts_with('~') && p.len() == 1 => number_of_items - 1,
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
        assert_eq!(parse_pattern(&history, "~")?, 1);

        Ok(())
    }
}
