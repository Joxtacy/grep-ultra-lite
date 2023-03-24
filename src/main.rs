use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use colored::Colorize;
use regex::{Captures, Regex};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "The pattern to look for")]
    pattern: String,
    #[arg(help = "The file(s) to search")]
    files: Vec<String>,
}

fn process_lines<T: BufRead + Sized>(reader: T, re: &Regex) -> Option<Vec<String>> {
    let mut found = false;
    let mut hits = vec![];

    for (i, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => match re.find(&line) {
                Some(_) => {
                    let line = re.replace_all(&line, |caps: &Captures| {
                        format!("{}", &caps[0].red().bold())
                    });

                    let hit = format!("{}: {}", (i + 1).to_string().green(), line);
                    hits.push(hit);
                    found = true;
                }
                None => (),
            },
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
        return Some(hits);
    } else {
        return None;
    }
}

fn render_results(hits: &Vec<String>) {
    for hit in hits {
        println!("{}", hit);
    }
}

fn main() {
    let cli = Cli::parse();

    let re = Regex::new(cli.pattern.as_str()).expect("Could not parse regex");

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();

        if let Some(hits) = process_lines(reader, &re) {
            render_results(&hits);
        }
    } else {
        for file in cli.files {
            let f = File::open(&file).expect(format!("Could not open file: {}", file).as_str());
            let reader = BufReader::new(f);

            if let Some(hits) = process_lines(reader, &re) {
                println!("{}", file.blue());
                render_results(&hits);
                println!();
            }
        }
    }
}
