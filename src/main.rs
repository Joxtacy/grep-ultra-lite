use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    sync::mpsc,
    thread,
};

use colored::Colorize;
use regex::{Captures, Regex, RegexBuilder};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "The pattern to look for")]
    pattern: String,
    #[arg(help = "The file(s) to search")]
    files: Vec<String>,
    #[arg(short, long, help = "Makes search case insensitive")]
    insensitive: bool,
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use colored::Colorize;
    use regex::Regex;

    use crate::process_lines;

    #[test]
    fn match_one() {
        let reader = BufReader::new("Hello test".as_bytes());
        let re = Regex::new("Hello").unwrap();

        let result = process_lines(reader, &re);

        let expected = format!("{}: {} {}", "1".green(), "Hello".red().bold(), "test");

        assert_eq!(result.unwrap()[0], expected);
    }

    #[test]
    fn no_matches() {
        let reader = BufReader::new("hi".as_bytes());
        let re = Regex::new("this is not gonna hit").unwrap();

        let result = process_lines(reader, &re);

        assert!(result.is_none());
    }
}

fn process_lines<T: BufRead + Sized>(reader: T, re: &Regex) -> Option<Vec<String>> {
    let mut found = false;
    let mut hits = vec![];

    for (i, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                if re.find(&line).is_some() {
                    let line = re.replace_all(&line, |caps: &Captures| {
                        format!("{}", &caps[0].red().bold())
                    });

                    let hit = format!("{}: {}", (i + 1).to_string().green(), line);
                    hits.push(hit);
                    found = true;
                }
            }
            Err(err) => {
                hits.push(format!(
                    "{}: Failed to parse line: {}",
                    (i + 1).to_string().green(),
                    err.to_string().red()
                ));
            }
        }
    }

    if found {
        Some(hits)
    } else {
        None
    }
}

fn render_results(hits: &Vec<String>) {
    for hit in hits {
        println!("{}", hit);
    }
}

fn main() {
    let cli = Cli::parse();

    let (tx, rx) = mpsc::channel();

    let re = RegexBuilder::new(cli.pattern.as_str())
        .case_insensitive(cli.insensitive)
        .build()
        .expect("Could not parse regex");

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();

        if let Some(hits) = process_lines(reader, &re) {
            render_results(&hits);
        }
    } else {
        for file in cli.files {
            let f = File::open(&file)
                .unwrap_or_else(|err| panic!("Could not open file: {}. Reason: {}", file, err));
            let re = re.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(f);

                if let Some(hits) = process_lines(reader, &re) {
                    tx.send((file, hits)).expect("Failed to send result");
                }
            });
        }

        drop(tx); // Drop the original one since its not being used.

        for result in rx {
            let (file, hits) = result;
            println!("{}", file.blue());
            render_results(&hits);
            println!();
        }
    }
}
