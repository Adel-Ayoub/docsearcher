use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::sequence::separated_pair;
use nom::IResult;

use anyhow::{Result, Context};

use crate::types::{FileType, Needle};

/// Parse a contact line in the format "search_term,metadata"
pub fn parse_contact(input: &str) -> IResult<&str, Needle> {
    let (input, _) = nom::character::complete::space0(input)?;
    let (input, result) = parse_contact_line(input)?;
    let (input, _) = nom::character::complete::space0(input)?;
    
    Ok((input, (result.0.trim(), result.1.trim())))
}

fn parse_contact_line(input: &str) -> IResult<&str, Needle> {
    separated_pair(is_not(","), char(','), is_not("\n"))(input)
}

/// Read search terms from a file
pub fn read_needles_from_file(path: &str) -> Result<Vec<(String, String)>> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open needles file: {}", path))?;
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| format!("Failed to read needles file: {}", path))?;
    
    read_needles_from_string(&content)
}

/// Read search terms from a byte slice
pub fn read_needles_from_mem(bytes: &[u8]) -> Result<Vec<(String, String)>> {
    let content = from_utf8(bytes)
        .with_context(|| "Failed to parse needles content as UTF-8")?;
    
    read_needles_from_string(content)
}

fn read_needles_from_string(content: &str) -> Result<Vec<(String, String)>> {
    let mut needles = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        match parse_contact(line) {
            Ok((_, needle)) => {
                needles.push((needle.0.to_string(), needle.1.to_string()));
            }
            Err(_) => {
                eprintln!("Warning: Failed to parse line {}: '{}'", line_num + 1, line);
            }
        }
    }
    
    if needles.is_empty() {
        return Err(anyhow::anyhow!("No valid search terms found in input"));
    }
    
    Ok(needles)
}

/// Parse file type from a file path
pub fn parse_filetype(file_path: &str) -> Result<FileType> {
    if file_path.ends_with(".docx") {
        Ok(FileType::Docx)
    } else if file_path.ends_with(".pdf") {
        Ok(FileType::Pdf)
    } else {
        Err(anyhow::anyhow!(
            "Unsupported file type. Only .docx and .pdf files are supported. Got: {}",
            file_path
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filetype() {
        assert_eq!(parse_filetype("document.docx").unwrap(), FileType::Docx);
        assert_eq!(parse_filetype("report.pdf").unwrap(), FileType::Pdf);
        assert!(parse_filetype("data.txt").is_err());
        assert!(parse_filetype("presentation").is_err());
    }

    #[test]
    fn test_parse_contact() {
        assert_eq!(
            parse_contact("Alice Johnson,alice.johnson@company.com"),
            Ok(("", ("Alice Johnson", "alice.johnson@company.com")))
        );
        assert_eq!(
            parse_contact("  Bob Smith  ,  bob.smith@enterprise.org  "),
            Ok(("", ("Bob Smith", "bob.smith@enterprise.org")))
        );
    }

    #[test]
    fn test_read_needles_from_string() {
        let input = "Alice Johnson,alice.johnson@company.com\nBob Smith,bob.smith@enterprise.org\n# Comment line\n\n";
        let result = read_needles_from_string(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("Alice Johnson".to_string(), "alice.johnson@company.com".to_string()));
        assert_eq!(result[1], ("Bob Smith".to_string(), "bob.smith@enterprise.org".to_string()));
    }
}
