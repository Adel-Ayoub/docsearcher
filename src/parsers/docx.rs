use anyhow::Result;
use colored::Colorize;
use std::{
    collections::HashSet,
    fs::File,
    io::{Cursor, Error, ErrorKind, Read},
    time::Instant,
};
use zip::ZipArchive;

use crate::utils::read_needles_from_file;
use crate::types::SearchResult;

enum AttributeType {
    OfficeDocument,
}

impl AttributeType {
    fn as_str(&self) -> &'static str {
        match self {
            AttributeType::OfficeDocument => {
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
            }
        }
    }
}

fn get_doc_name<R>(archive: &mut ZipArchive<R>) -> Option<String>
where
    R: std::io::Seek,
    R: std::io::Read,
{
    let mut doc_name = None;
    let names: Vec<_> = archive.file_names().collect();
    println!("Found {} files in archive, {:?}", names.len(), names);
    let mut rels = archive.by_name("_rels/.rels").unwrap();
    let mut rels_buffer = String::new();
    rels.read_to_string(&mut rels_buffer).unwrap();

    let rel_xml = roxmltree::Document::parse(&rels_buffer).unwrap();

    for elem in rel_xml.descendants() {
        'outer: for attr in elem.attributes() {
            if attr.name() == "Type" && attr.value() == AttributeType::OfficeDocument.as_str() {
                if let Some(target) = elem.attribute("Target") {
                    doc_name = Some(target.to_owned());
                }
                break 'outer;
            }
        }
    }

    doc_name
}

pub fn parse_from_mem(
    needle_bytes: &[u8],
    haystack_bytes: &[u8],
) -> Result<HashSet<SearchResult>> {
    let needles = crate::utils::read_needles_from_mem(needle_bytes)?;
    println!("Searching across {} contacts", needles.len());

    let haystack_reader = Cursor::new(haystack_bytes);
    let mut archive = ZipArchive::new(haystack_reader)?;

    parse(&needles, &mut archive)
}

pub fn parse_from_path(needle_path: &str, file_path: &str) -> Result<HashSet<SearchResult>> {
    let start = Instant::now();
    let needles = read_needles_from_file(needle_path)?;
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
    let file: File = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;
    println!(
        "{}",
        format!("Opened archive in {} ms", start.elapsed().as_millis()).blue()
    );
    parse(&needles, &mut archive)
}

fn parse<R>(
    needles: &[(String, String)],
    archive: &mut ZipArchive<R>,
) -> Result<HashSet<SearchResult>>
where
    R: std::io::Seek,
    R: std::io::Read,
{
    let start = Instant::now();
    println!("{}", format!("Creating haystack from document...",).blue());

    let doc_name = get_doc_name(archive)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Could not find document name"))?;
    println!("Found document name: {}", doc_name);

    let mut document = archive
        .by_name(&doc_name)
        .map_err(|_| Error::new(ErrorKind::NotFound, "Could not find document in archive"))?;

    let mut buffer = String::new();
    document.read_to_string(&mut buffer).map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            "Failed to write document to buffer",
        )
    })?;

    let doc = roxmltree::Document::parse(&buffer)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Could not parse XML tree"))?;

    let root = doc
        .root()
        .first_child()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Could not find root node"))?;

    let body = root
        .first_element_child()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Root node is empty"))?;

    let haystack = body
        .descendants()
        .filter(|elem| elem.has_tag_name("p"))
        .fold(Vec::new(), |mut acc, elem| {
            elem.descendants()
                .filter(|elem| elem.has_tag_name("r"))
                .for_each(|elem| {
                    elem.descendants()
                        .filter(|elem| elem.has_tag_name("t"))
                        .for_each(|elem| {
                            elem.text().and_then(|text| {
                                return Some(acc.push(text));
                            });
                        });
                });

            acc
        });
    println!(
        "{}",
        format!(
            "Haystack created. Extracted {} lines from document in {} ms",
            haystack.len(),
            start.elapsed().as_millis()
        )
        .blue()
    );

    println!("{}", "Starting search...".blue());
    let start = Instant::now();
    let matches = haystack.iter().fold(HashSet::new(), |mut acc, substack| {
        needles
            .iter()
            .filter(|needle| substack.contains(&needle.0))
            .for_each(|needle| {
                acc.insert((needle.0.clone(), needle.1.clone()));
            });

        acc
    });
    println!(
        "{}",
        format!("Search completed in {} ms", start.elapsed().as_millis()).blue()
    );

    println!("{}", format!("Found {} matches:", matches.len(),).green());
    matches
        .iter()
        .enumerate()
        .for_each(|(i, match_)| println!("{}", format!("{}: {:?}", i + 1, match_).green()));

    Ok(matches)
}
