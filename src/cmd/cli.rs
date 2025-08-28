use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Input, Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::path::PathBuf;
use walkdir::WalkDir;
use glob::glob;

use crate::{
    types::{FileType, SearchResult},
    utils::{parse_filetype, read_needles_from_file},
    parsers::{parse_docx_from_path, parse_pdf_from_path},
    cmd::tui::TuiApp,
};

#[derive(Parser)]
#[command(name = "DocSearcher")]
#[command(about = "A fast document search tool for PDF and DOCX files")]
#[command(version)]
#[command(propagate_version = true)]
pub struct EnhancedCli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to file containing search terms (CSV format: term,metadata)
    #[arg(short, long)]
    needles: Option<PathBuf>,

    /// Path to document file (.docx or .pdf)
    #[arg(short, long)]
    document: Option<PathBuf>,

    /// Enable interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Enable TUI mode
    #[arg(short, long)]
    tui: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long)]
    quiet: bool,

    /// Case sensitive search
    #[arg(long)]
    case_sensitive: bool,

    /// Whole word matching
    #[arg(long)]
    whole_word: bool,

    /// Output format (text, json, csv, html)
    #[arg(short, long, default_value = "text")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive search mode
    Interactive,
    
    /// TUI mode with modern interface
    Tui,
    
    /// Search in a specific document
    Search {
        /// Path to file containing search terms
        needles: PathBuf,
        
        /// Path to document file
        document: PathBuf,
        
        /// Output format (text, json, csv, html)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Case sensitive search
        #[arg(long)]
        case_sensitive: bool,
        
        /// Whole word matching
        #[arg(long)]
        whole_word: bool,
    },
    
    /// Batch process multiple files
    Batch {
        /// Directory containing documents
        #[arg(short, long)]
        directory: String,
        
        /// Path to needles file
        #[arg(short, long)]
        needles_file: String,
        
        /// File pattern (e.g., "*.pdf", "*.docx")
        #[arg(short, long, default_value = "*.*")]
        pattern: String,
        
        /// Recursive search
        #[arg(short, long)]
        recursive: bool,
        
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Validate files without searching
    Validate {
        /// Path to needles file
        needles: PathBuf,
        
        /// Path to document file
        document: PathBuf,
    },
    
    /// Show file information
    Info {
        /// Path to document file
        file: PathBuf,
    },
}

pub struct CliApp {
    cli: EnhancedCli,
}

impl CliApp {
    pub fn new() -> Self {
        Self {
            cli: EnhancedCli::parse(),
        }
    }

    pub fn run() -> Result<()> {
        let app = Self::new();
        
        match app.cli.command.as_ref() {
            Some(Commands::Interactive) => Self::run_interactive(),
            Some(Commands::Tui) => Self::run_tui(),
            Some(Commands::Search { needles, document, format: _format, case_sensitive: _case_sensitive, whole_word: _whole_word }) => {
                Self::run_search(needles, document, *_case_sensitive, *_whole_word, _format)
            }
            Some(Commands::Batch { directory, needles_file, pattern: _pattern, recursive: _recursive, format }) => {
                let directory_path = PathBuf::from(directory);
                let needles_path = PathBuf::from(needles_file);
                Self::run_batch(&needles_path, &directory_path, false, false, &format)
            }
            Some(Commands::Validate { needles, document }) => {
                Self::run_validate(Some(&needles), Some(&document))
            }
            Some(Commands::Info { file: _file }) => {
                Self::run_info()
            }
            None => {
                if app.cli.tui {
                    Self::run_tui()
                } else if app.cli.interactive {
                    Self::run_interactive()
                } else if let (Some(needles), Some(document)) = (&app.cli.needles, &app.cli.document) {
                    Self::run_search(&needles, &document, app.cli.case_sensitive, app.cli.whole_word, &app.cli.format)
                } else {
                    Self::show_help();
                    Ok(())
                }
            }
        }
    }

    fn run_interactive() -> Result<()> {
        Self::show_startup_logo();
        
        println!("{}", "Interactive Mode".bold().blue());
        println!("{}", "=================".blue());
        
        let search_terms = Self::get_search_terms_interactive()?;
        let target_files = Self::get_target_files_interactive()?;
        let (_case_sensitive, _whole_word) = Self::get_search_options_interactive()?;
        
        println!("\n{}", "Starting search...".green());
        
        for (term, metadata) in &search_terms {
            println!("Searching for: {} ({})", term.cyan(), metadata.yellow());
            
            for file_path in &target_files {
                if let Ok(file_type) = parse_filetype(&file_path.to_string_lossy()) {
                    let results = match file_type {
                        FileType::Docx => parse_docx_from_path("contacts.csv", &file_path.to_string_lossy())?,
                        FileType::Pdf => parse_pdf_from_path("contacts.csv", &file_path.to_string_lossy())?,
                    };
                    
                    if !results.is_empty() {
                        println!("  Found {} matches in {}", results.len().to_string().green(), file_path.display());
                        for (found_term, found_metadata) in results {
                            println!("    {} -> {}", found_term.cyan(), found_metadata.yellow());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn run_tui() -> Result<()> {
        let mut tui_app = TuiApp::default();
        tui_app.run()
    }
    
    fn run_search(needles: &PathBuf, document: &PathBuf, _case_sensitive: bool, _whole_word: bool, format: &str) -> Result<()> {
        println!("{}", "Search Mode".bold().blue());
        println!("{}", "=============".blue());
        
        if !needles.exists() {
            return Err(anyhow::anyhow!("Needles file not found: {}", needles.display()));
        }
        
        if !document.exists() {
            return Err(anyhow::anyhow!("Document file not found: {}", document.display()));
        }
        
        let search_terms = read_needles_from_file(&needles.to_string_lossy())?;
        let file_type = parse_filetype(&document.to_string_lossy())?;
        
        println!("Searching for {} terms in {}", search_terms.len(), document.display());
        
        let results = match file_type {
            FileType::Docx => parse_docx_from_path(&needles.to_string_lossy(), &document.to_string_lossy())?,
            FileType::Pdf => parse_pdf_from_path(&needles.to_string_lossy(), &document.to_string_lossy())?,
        };
        
        Self::display_results(&results, format, std::time::Duration::from_secs(0))
    }
    
    fn run_batch(needles: &PathBuf, directory: &PathBuf, case_sensitive: bool, whole_word: bool, format: &str) -> Result<()> {
        println!("{}", "Batch Mode".bold().blue());
        println!("{}", "===========".blue());
        
        if !needles.exists() {
            return Err(anyhow::anyhow!("Needles file not found: {}", needles.display()));
        }
        
        if !directory.exists() || !directory.is_dir() {
            return Err(anyhow::anyhow!("Directory not found: {}", directory.display()));
        }
        
        let search_terms = read_needles_from_file(&needles.to_string_lossy())?;
        let files = Self::scan_directory(directory, "*.*", false)?;
        
        println!("Found {} files to process", files.len());
        
        Self::run_batch_search(&search_terms, &files, case_sensitive, whole_word, format)
    }
    
    fn run_validate(needles: Option<&PathBuf>, document: Option<&PathBuf>) -> Result<()> {
        println!("{}", "Validation Mode".bold().blue());
        println!("{}", "=================".blue());
        
        let needles_valid = Self::validate_needles_file(needles);
        let document_valid = Self::validate_document_file(document);
        
        println!("{}", "Validation Results:".bold());
        println!("Needles file: {}", if needles_valid { "✓ Valid".green() } else { "✗ Invalid".red() });
        println!("Document file: {}", if document_valid { "✓ Valid".green() } else { "✗ Invalid".red() });
        
        Ok(())
    }
    
    fn run_info() -> Result<()> {
        println!("{}", "File Information".bold().blue());
        println!("{}", "==================".blue());
        
        let file = Self::get_document_path_interactive()?;
        if !file.exists() {
            eprintln!("{}", format!("File not found: {}", file.display()).red());
            return Ok(());
        }
        
        if let Ok(file_type) = parse_filetype(&file.to_string_lossy()) {
            println!("File: {}", file.display());
            println!("Type: {}", match file_type {
                FileType::Docx => "DOCX Document".blue(),
                FileType::Pdf => "PDF Document".red(),
            });
            println!("Size: {} bytes", file.metadata()?.len());
        } else {
            eprintln!("{}", "Unsupported file type".red());
        }
        
        Ok(())
    }

    fn get_search_terms_interactive() -> Result<Vec<(String, String)>> {
        let options = &[
            "Enter search terms manually",
            "Import from file",
            "Use sample terms",
        ];
        
        let choice = Select::new()
            .with_prompt("How would you like to input search terms?")
            .default(0)
            .items(options)
            .interact()?;
        
        match choice {
            0 => {
                let terms_input: String = Input::new()
                    .with_prompt("Enter search terms (separated by commas, e.g., term1,metadata1,term2,metadata2)")
                    .interact_text()?;
                
                Ok(terms_input.split(',')
                    .map(|s| {
                        let parts: Vec<&str> = s.trim().splitn(2, ',').collect();
                        if parts.len() == 2 {
                            (parts[0].to_string(), parts[1].to_string())
                        } else {
                            (parts[0].to_string(), "".to_string())
                        }
                    })
                    .collect())
            }
            1 => {
                let file_path: String = Input::new()
                    .with_prompt("Enter path to needles file")
                    .default("contacts.csv".to_string())
                    .interact_text()?;
                
                let needles = read_needles_from_file(&file_path)?;
                Ok(needles)
            }
            2 => {
                Ok(vec![
                    ("Alice Johnson".to_string(), "".to_string()),
                    ("Bob Smith".to_string(), "".to_string()),
                    ("Carol Davis".to_string(), "".to_string()),
                ])
            }
            _ => unreachable!(),
        }
    }

    fn get_target_files_interactive() -> Result<Vec<PathBuf>> {
        let options = &[
            "Select individual files",
            "Select directory with pattern",
            "Use current directory",
        ];
        
        let choice = Select::new()
            .with_prompt("How would you like to select target files?")
            .default(0)
            .items(options)
            .interact()?;
        
        match choice {
            0 => {
                let files_input: String = Input::new()
                    .with_prompt("Enter file paths (separated by spaces)")
                    .interact_text()?;
                
                Ok(files_input.split_whitespace()
                    .map(|s| PathBuf::from(s.trim()))
                    .collect())
            }
            1 => {
                let dir_path: String = Input::new()
                    .with_prompt("Enter directory path")
                    .interact_text()?;
                
                let pattern: String = Input::new()
                    .with_prompt("Enter file pattern (e.g., *.pdf)")
                    .default("*.pdf".to_string())
                    .interact_text()?;
                
                let files = Self::scan_directory(&PathBuf::from(dir_path.clone()), &pattern, false)?;
                if files.is_empty() {
                    return Err(anyhow::anyhow!("No files found in directory: {}", dir_path));
                }
                let file = Select::new()
                    .with_prompt("Select document file")
                    .items(&files.iter().map(|f| f.to_string_lossy().to_string()).collect::<Vec<_>>())
                    .interact()?;
                Ok(vec![files[file].clone()])
            }
            2 => {
                let files = Self::scan_directory(&PathBuf::from("."), "*.*", false)?;
                Ok(files)
            }
            _ => unreachable!(),
        }
    }

    fn get_search_options_interactive() -> Result<(bool, bool)> {
        let case_sensitive = Confirm::new()
            .with_prompt("Enable case sensitive search?")
            .default(false)
            .interact()?;
        
        let whole_word = Confirm::new()
            .with_prompt("Enable whole word matching?")
            .default(false)
            .interact()?;
        
        Ok((case_sensitive, whole_word))
    }

    fn get_document_path_interactive() -> Result<PathBuf> {
        let options = &[
            "Enter document path manually",
            "Select from current directory",
        ];
        
        let choice = Select::new()
            .with_prompt("How would you like to select the document file?")
            .default(0)
            .items(options)
            .interact()?;
        
        match choice {
            0 => {
                let file_path: String = Input::new()
                    .with_prompt("Enter document path")
                    .interact_text()?;
                Ok(PathBuf::from(file_path.trim()))
            }
            1 => {
                let dir_path: String = Input::new()
                    .with_prompt("Enter directory path")
                    .interact_text()?;
                let pattern: String = Input::new()
                    .with_prompt("Enter file pattern (e.g., *.pdf)")
                    .default("*.pdf".to_string())
                    .interact_text()?;
                let files = Self::scan_directory(&PathBuf::from(dir_path.clone()), &pattern, false)?;
                if files.is_empty() {
                    return Err(anyhow::anyhow!("No files found in directory: {}", dir_path));
                }
                let file = Select::new()
                    .with_prompt("Select document file")
                    .items(&files.iter().map(|f| f.to_string_lossy().to_string()).collect::<Vec<_>>())
                    .interact()?;
                Ok(files[file].clone())
            }
            _ => unreachable!(),
        }
    }

    fn scan_directory(directory: &PathBuf, pattern: &str, recursive: bool) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if recursive {
            for entry in WalkDir::new(directory)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path().to_string_lossy();
                if glob::Pattern::new(pattern).unwrap().matches(&path) {
                    files.push(PathBuf::from(path.as_ref()));
                }
            }
        } else {
            let search_pattern = format!("{}/{}", directory.display(), pattern);
            for entry in glob(&search_pattern)? {
                if let Ok(path) = entry {
                    if path.is_file() {
                        files.push(path.to_string_lossy().to_string().into());
                    }
                }
            }
        }
        
        // Filter by supported file types
        files.retain(|file| {
            file.ends_with(".pdf") || file.ends_with(".docx")
        });
        
        Ok(files)
    }

    fn run_batch_search(_search_terms: &[(String, String)], files: &[PathBuf], _case_sensitive: bool, _whole_word: bool, format: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let total_files = files.len() as u64;
        
        // Create multi-progress bar
        let multi_progress = MultiProgress::new();
        let overall_progress = multi_progress.add(ProgressBar::new(total_files));
        overall_progress.set_style(
            ProgressStyle::default_bar()
                .template("Overall: [{bar:40.cyan/blue}] {pos}/{len} files")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ ")
        );
        
        let mut all_results = Vec::new();
        let mut files_with_matches = 0;
        
        for (_i, file_path) in files.iter().enumerate() {
            overall_progress.set_message(format!("Processing: {}", file_path.display()));
            
            // Process individual file
            if let Ok(file_type) = parse_filetype(&file_path.to_string_lossy()) {
                let results = match file_type {
                    FileType::Docx => parse_docx_from_path("contacts.csv", &file_path.to_string_lossy())?,
                    FileType::Pdf => parse_pdf_from_path("contacts.csv", &file_path.to_string_lossy())?,
                };
                
                if !results.is_empty() {
                    files_with_matches += 1;
                    for (term, metadata) in results {
                        all_results.push((term, metadata, file_path.clone()));
                    }
                }
            }
            
            overall_progress.inc(1);
        }
        
        overall_progress.finish_with_message("Batch processing completed!");
        
        let duration = start.elapsed();
        
        // Display batch results
        Self::display_batch_results(&all_results, format, duration, files.len(), files_with_matches)
    }

    fn validate_needles_file(path: Option<&PathBuf>) -> bool {
        if let Some(path) = path {
            if !path.exists() {
                return false;
            }
            
            match read_needles_from_file(&path.to_string_lossy()) {
                Ok(needles) => !needles.is_empty(),
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn validate_document_file(path: Option<&PathBuf>) -> bool {
        if let Some(path) = path {
            if !path.exists() {
                return false;
            }
            
            parse_filetype(&path.to_string_lossy()).is_ok()
        } else {
            false
        }
    }

    fn display_results(matches: &std::collections::HashSet<SearchResult>, format: &str, duration: std::time::Duration) -> Result<()> {
        println!("\n{}", "=".repeat(50).blue());
        println!("{}", "SEARCH RESULTS".blue().bold());
        println!("{}", "=".repeat(50).blue());
        
        // Show search options
        println!("Search Options:");
        println!("  Case sensitive: {}", "N/A".yellow());
        println!("  Whole word: {}", "N/A".yellow());
        println!();
        
        match format.to_lowercase().as_str() {
            "json" => Self::display_json_results(matches)?,
            "csv" => Self::display_csv_results(matches)?,
            "html" => Self::display_html_results(matches)?,
            _ => Self::display_text_results(matches),
        }
        
        println!("{}", "=".repeat(50).blue());
        println!("{}", format!("Search completed in {} ms", duration.as_millis()).italic());
        println!("{}", format!("Found {} matches", matches.len()).green().bold());
        
        Ok(())
    }

    fn display_batch_results(results: &[(String, String, PathBuf)], format: &str, duration: std::time::Duration, total_files: usize, files_with_matches: usize) -> Result<()> {
        println!("\n{}", "=".repeat(60).blue());
        println!("{}", "BATCH SEARCH RESULTS".blue().bold());
        println!("{}", "=".repeat(60).blue());
        
        println!("Summary:");
        println!("  Total files processed: {}", total_files);
        println!("  Files with matches: {}", files_with_matches);
        println!("  Total matches found: {}", results.len());
        println!();
        
        match format.to_lowercase().as_str() {
            "json" => Self::display_batch_json_results(results)?,
            "csv" => Self::display_batch_csv_results(results)?,
            "html" => Self::display_batch_html_results(results)?,
            _ => Self::display_batch_text_results(results),
        }
        
        println!("{}", "=".repeat(60).blue());
        println!("{}", format!("Batch processing completed in {} ms", duration.as_millis()).italic());
        
        Ok(())
    }

    fn display_text_results(matches: &std::collections::HashSet<SearchResult>) {
        if matches.is_empty() {
            println!("{}", "No matches found.".yellow());
            return;
        }
        
        for (i, (term, metadata)) in matches.iter().enumerate() {
            println!("  {}: {} → {}", i + 1, term.blue(), metadata.green());
        }
    }

    fn display_batch_text_results(results: &[(String, String, PathBuf)]) {
        if results.is_empty() {
            println!("{}", "No matches found in any files.".yellow());
            return;
        }
        
        for (i, (term, metadata, file)) in results.iter().enumerate() {
            println!("  {}: {} → {} [{}]", i + 1, term.blue(), metadata.green(), file.display());
        }
    }

    fn display_json_results(matches: &std::collections::HashSet<SearchResult>) -> Result<()> {
        let results: Vec<serde_json::Value> = matches
            .iter()
            .map(|(term, metadata)| {
                serde_json::json!({
                    "term": term,
                    "metadata": metadata
                })
            })
            .collect();
        
        println!("{}", serde_json::to_string_pretty(&results)?);
        Ok(())
    }

    fn display_batch_json_results(results: &[(String, String, PathBuf)]) -> Result<()> {
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|(term, metadata, file)| {
                serde_json::json!({
                    "term": term,
                    "metadata": metadata,
                    "file": file.to_string_lossy()
                })
            })
            .collect();
        
        println!("{}", serde_json::to_string_pretty(&results_json)?);
        Ok(())
    }

    fn display_csv_results(matches: &std::collections::HashSet<SearchResult>) -> Result<()> {
        println!("term,metadata");
        for (term, metadata) in matches {
            println!("{},{},", term, metadata);
        }
        Ok(())
    }

    fn display_batch_csv_results(results: &[(String, String, PathBuf)]) -> Result<()> {
        println!("term,metadata,file");
        for (term, metadata, file) in results {
            println!("{},{},{}", term, metadata, file.to_string_lossy());
        }
        Ok(())
    }

    fn display_html_results(matches: &std::collections::HashSet<SearchResult>) -> Result<()> {
        println!("<!DOCTYPE html>");
        println!("<html><head><title>DocSearcher Results</title></head><body>");
        println!("<h1>Search Results</h1>");
        println!("<table border='1'><tr><th>Term</th><th>Metadata</th></tr>");
        
        for (term, metadata) in matches {
            println!("<tr><td>{}</td><td>{}</td></tr>", term, metadata);
        }
        
        println!("</table></body></html>");
        Ok(())
    }

    fn display_batch_html_results(results: &[(String, String, PathBuf)]) -> Result<()> {
        println!("<!DOCTYPE html>");
        println!("<html><head><title>DocSearcher Batch Results</title></head><body>");
        println!("<h1>Batch Search Results</h1>");
        println!("<table border='1'><tr><th>Term</th><th>Metadata</th><th>File</th></tr>");
        
        for (term, metadata, file) in results {
            println!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>", term, metadata, file.to_string_lossy());
        }
        
        println!("</table></body></html>");
        Ok(())
    }

    fn show_help() {
        println!("{}", "DocSearcher - Document Search Tool".blue().bold());
        println!();
        println!("Usage:");
        println!("  docsearcher <needles_file> <document_file>");
        println!("  docsearcher --interactive");
        println!("  docsearcher --tui");
        println!("  docsearcher search <needles_file> <document_file>");
        println!("  docsearcher batch <directory> <needles_file>");
        println!("  docsearcher validate <needles_file> <document_file>");
        println!("  docsearcher info <file>");
        println!();
        println!("Examples:");
        println!("  docsearcher contacts.csv document.docx");
        println!("  docsearcher --interactive");
        println!("  docsearcher --tui");
        println!("  docsearcher search contacts.csv report.pdf --format json");
        println!("  docsearcher batch ./documents contacts.csv --pattern *.pdf");
        println!("  docsearcher validate contacts.csv document.docx");
        println!("  docsearcher info report.pdf");
        println!();
        println!("For more help, run: docsearcher --help");
    }

    fn show_startup_logo() {
        let logo = r#"
 ____             ____                      _               
|  _ \  ___   ___/ ___|  ___  __ _ _ __ ___| |__   ___ _ __ 
| | | |/ _ \ / __\___ \ / _ \/ _` | '__/ __| '_ \ / _ \ '__|
| |_| | (_) | (__ ___) |  __/ (_| | | | (__| | | |  __/ |  
|____/ \___/ \___|____/ \___|\__,_|_|  \___|_| |_|\___|_|  
"#;
        println!("{}", logo);
        println!();
    }
}
