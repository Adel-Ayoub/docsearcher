use anyhow::{Context, Result};
use colored::Colorize;
use std::{
    collections::HashSet,
    time::Instant,
};

use crate::utils::read_needles_from_file;
use crate::types::SearchResult;

pub fn parse_from_mem(
    needle_bytes: &[u8],
    haystack_bytes: &[u8],
) -> Result<HashSet<SearchResult>> {
    let needles = crate::utils::read_needles_from_mem(needle_bytes)?;
    println!("Searching across {} contacts", needles.len());

    parse(&needles, haystack_bytes)
}

pub fn parse_from_path(
    needles_path: &str,
    haystack_path: &str,
) -> Result<HashSet<SearchResult>> {
    let start = Instant::now();
    let needles = read_needles_from_file(needles_path)?;
    println!(
        "{}",
        format!(
            "Read {} contacts in {} ms",
            needles.len(),
            start.elapsed().as_millis()
        )
        .blue()
    );

    let start = Instant::now();
    let text = pdf_extract::extract_text(haystack_path)?;
    println!(
        "{}",
        format!("Extracted text in {} ms", start.elapsed().as_millis()).blue()
    );

    println!("{}", "Starting search...".blue());
    let start = Instant::now();
    let matches = text.lines().fold(HashSet::new(), |mut acc, line| {
        needles
            .iter()
            .filter(|n| line.contains(&n.0))
            .for_each(|n| {
                acc.insert((n.0.clone(), n.1.clone()));
            });
        acc
    });
    println!(
        "{}",
        format!("Search completed in {} ms", start.elapsed().as_millis()).blue()
    );

    Ok(matches)
}

fn parse(needles: &[(String, String)], haystack_bytes: &[u8]) -> Result<HashSet<SearchResult>> {
    println!("{}", format!("Starting extracting text from pdf...").blue());
    let start = Instant::now();
    let haystack = pdf_extract::extract_text_from_mem(&haystack_bytes).with_context(|| {
        format!(
            "Failed to extract text from pdf: {}",
            String::from_utf8_lossy(haystack_bytes)
        )
    })?;
    let duration = start.elapsed();
    println!(
        "{}",
        format!("Extracting text from pdf took {} ms", duration.as_millis()).italic()
    );

    println!("{}", format!("Starting search...").blue());
    let start = Instant::now();
    let matches = haystack.lines().filter(|line| line.trim().len() > 0).fold(
        HashSet::new(),
        |mut acc, line| {
            needles.iter().filter(|n| line.contains(&n.0)).for_each(|n| {
                acc.insert((n.0.clone(), n.1.clone()));
            });

            acc
        },
    );
    let duration = start.elapsed();
    println!(
        "{}",
        format!("Searching took {} ms", duration.as_millis()).italic()
    );

    println!("{}", format!("Found {} matches", matches.len()).green());
    Ok(matches)
}
