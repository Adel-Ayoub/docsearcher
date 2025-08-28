use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs,
    },
    Frame, Terminal,
};
use std::{
    io::stdout,
    time::Duration,
};

use crate::{
    types::{FileType, SearchResult},
    utils::{parse_filetype},
    parsers::{parse_docx_from_path, parse_pdf_from_path},
};

pub struct TuiApp {
    pub current_tab: usize,
    pub search_terms: Vec<String>,
    pub selected_files: Vec<String>,
    pub search_results: Vec<SearchResult>,
    pub is_searching: bool,
    pub search_progress: f32,
    pub current_file: String,
    pub files_processed: usize,
    pub total_files: usize,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            current_tab: 0,
            search_terms: Vec::new(),
            selected_files: Vec::new(),
            search_results: Vec::new(),
            is_searching: false,
            search_progress: 0.0,
            current_file: String::new(),
            files_processed: 0,
            total_files: 0,
        }
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) -> Result<()> {
        // Show startup logo
        self.show_startup_logo()?;
        
        // Clear screen before starting TUI
        execute!(stdout(), Clear(ClearType::All))?;
        
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, Hide)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), Show)?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err);
        }

        Ok(())
    }
    
    fn show_startup_logo(&self) -> Result<()> {
        let logo = r#"
DocSearcher
===========
"#;
        println!("{}", logo);
        
        // Give user a moment to see the logo
        std::thread::sleep(Duration::from_millis(500));
        
        Ok(())
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                if let KeyCode::Char('h') = key.code {
                    self.current_tab = (self.current_tab + 1) % 4;
                }
                if let KeyCode::Char('l') = key.code {
                    self.current_tab = if self.current_tab == 0 { 3 } else { self.current_tab - 1 };
                }
                if let KeyCode::Char('s') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.start_search()?;
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),  // Header
                    Constraint::Length(3),  // Tabs
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Status bar
                ]
                .as_ref(),
            )
            .split(size);

        // Header
        self.draw_header(f, chunks[0]);
        
        // Tabs
        self.draw_tabs(f, chunks[1]);
        
        // Content based on current tab
        match self.current_tab {
            0 => self.draw_search_tab(f, chunks[2]),
            1 => self.draw_files_tab(f, chunks[2]),
            2 => self.draw_results_tab(f, chunks[2]),
            3 => self.draw_settings_tab(f, chunks[2]),
            _ => unreachable!(),
        }
        
        // Status bar
        self.draw_status_bar(f, chunks[3]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled("DocSearcher", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::raw(" v1.0"),
        ]);
        
        let paragraph = Paragraph::new(title)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }

    fn draw_tabs(&self, f: &mut Frame, area: Rect) {
        let titles = vec!["Search", "Files", "Results", "Settings"];
        let tabs = titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Line::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(rest, Style::default().fg(Color::Gray)),
                ])
            })
            .collect();

        let tabs = Tabs::new(tabs)
            .select(self.current_tab)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        f.render_widget(tabs, area);
    }

    fn draw_search_tab(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ].as_ref())
            .split(area);

        // Search terms input
        let search_terms = if self.search_terms.is_empty() {
            "Enter search terms (one per line)...".to_string()
        } else {
            self.search_terms.join("\n")
        };
        
        let search_input = Paragraph::new(search_terms)
            .block(Block::default().title("Search Terms").borders(Borders::ALL));
        f.render_widget(search_input, chunks[0]);

        // File selection
        let files_display = if self.selected_files.is_empty() {
            "No files selected...".to_string()
        } else {
            format!("{} files selected", self.selected_files.len())
        };
        
        let files_input = Paragraph::new(files_display)
            .block(Block::default().title("Target Files").borders(Borders::ALL));
        f.render_widget(files_input, chunks[1]);

        // Search button
        let search_button = Paragraph::new("Press Ctrl+S to start search")
            .block(Block::default().title("Actions").borders(Borders::ALL));
        f.render_widget(search_button, chunks[2]);
    }

    fn draw_files_tab(&self, f: &mut Frame, area: Rect) {
        let files: Vec<ListItem> = self.selected_files
            .iter()
            .map(|file| {
                let extension = file.split('.').last().unwrap_or("");
                let indicator = match extension.to_lowercase().as_str() {
                    "pdf" => "[PDF]",
                    "docx" => "[DOCX]",
                    _ => "[UNK]",
                };
                
                ListItem::new(vec![Line::from(vec![
                    Span::styled(indicator, Style::default().fg(Color::Blue)),
                    Span::raw(" "),
                    Span::raw(file),
                ])])
            })
            .collect();

        let files_list = List::new(files)
            .block(Block::default().title("Selected Files").borders(Borders::ALL));
        f.render_widget(files_list, area);
    }

    fn draw_results_tab(&self, f: &mut Frame, area: Rect) {
        if self.search_results.is_empty() {
            let no_results = Paragraph::new("No search results yet. Run a search to see results here.")
                .block(Block::default().title("Search Results").borders(Borders::ALL));
            f.render_widget(no_results, area);
            return;
        }

        let results: Vec<Row> = self.search_results
            .iter()
            .map(|result| {
                Row::new(vec![
                    result.0.clone(),
                    result.1.clone(),
                    "Match".to_string(),
                ])
            })
            .collect();

        let table = Table::new(results)
            .header(Row::new(vec!["Term", "Metadata", "Status"]))
            .block(Block::default().title("Search Results").borders(Borders::ALL))
            .widths(&[
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ]);

        f.render_widget(table, area);
    }

    fn draw_settings_tab(&self, f: &mut Frame, area: Rect) {
        let settings_text = vec![
            "Keyboard Shortcuts:",
            "  h/l - Navigate tabs",
            "  Ctrl+S - Start search",
            "  q - Quit",
            "",
            "Search Options:",
            "  Case sensitive: false",
            "  Whole word: false",
            "  Pattern matching: false",
        ];

        let settings = Paragraph::new(settings_text.join("\n"))
            .block(Block::default().title("Settings").borders(Borders::ALL));
        f.render_widget(settings, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.is_searching {
            format!(
                "Searching: {} ({:.1}%) - {} of {} files processed",
                self.current_file,
                self.search_progress * 100.0,
                self.files_processed,
                self.total_files
            )
        } else {
            "Ready - Press 'h' for help, 'q' to quit".to_string()
        };

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(status, area);
    }

    fn start_search(&mut self) -> Result<()> {
        if self.search_terms.is_empty() || self.selected_files.is_empty() {
            return Ok(());
        }

        self.is_searching = true;
        self.files_processed = 0;
        self.total_files = self.selected_files.len();
        self.search_results.clear();

        for (i, file_path) in self.selected_files.iter().enumerate() {
            self.current_file = file_path.clone();
            self.files_processed = i;
            self.search_progress = i as f32 / self.total_files as f32;

            // Process the file
            if let Ok(file_type) = parse_filetype(file_path) {
                let result = match file_type {
                    FileType::Docx => parse_docx_from_path("contacts.csv", file_path),
                    FileType::Pdf => parse_pdf_from_path("contacts.csv", file_path),
                };

                if let Ok(matches) = result {
                    for (term, metadata) in matches {
                        self.search_results.push((term, metadata));
                    }
                }
            }

            // Small delay to show progress
            std::thread::sleep(Duration::from_millis(100));
        }

        self.is_searching = false;
        self.search_progress = 1.0;
        self.files_processed = self.total_files;

        Ok(())
    }
}

pub fn show_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg}: [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ ")
    );
    pb.set_message(message.to_string());
    pb
}
