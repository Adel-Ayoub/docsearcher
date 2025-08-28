# DocSearcher

>**W.I.P** - This project is actively under development

## A powerful document search tool in Rust.

## Installation

```sh
# Clone
git clone https://github.com/Adel-Ayoub/docsearcher.git
cd docsearcher

# Build
cargo build --release

# Run
cargo run -- --help
```

---

## Requirements

- Rust (latest stable)
- Cargo package manager

---

## Features

### Completed Features

#### Core Search Functionality
- **PDF Parsing**: Full text extraction and search using `pdf-extract`
- **DOCX Parsing**: Microsoft Word document parsing and search
- **Fast Search**: Efficient text search with HashSet-based matching
- **Batch Processing**: Process multiple files simultaneously
- **Interactive Mode**: User-friendly interactive search interface

#### Command Line Interface
- **Search Mode**: Direct file-to-file search with detailed results
- **Batch Mode**: Process entire directories with pattern matching
- **Validation**: File type validation and error checking
- **Help System**: Comprehensive help and usage information
- **Version Info**: Built-in version and project information

#### Terminal User Interface (TUI)
- **Full-Screen Interface**: Modern terminal-based user interface
- **Tabbed Navigation**: Organized interface with multiple tabs
- **Real-time Search**: Interactive search with live results
- **File Browser**: Integrated file system navigation
- **Clean Layout**: Professional, clutter-free design

#### Advanced Features
- **Multiple File Types**: Support for PDF and DOCX documents
- **Flexible Search**: Custom search term definitions with categories
- **Progress Tracking**: Visual progress indicators for long operations
- **Error Handling**: Robust error handling with user-friendly messages
- **Performance**: Optimized for large document processing

### Planned Features
- [ ] Additional file format support
- [ ] Memory optimization and large document handling improvements
- [ ] Enhanced search algorithms and relevance ranking
- [ ] Fuzzy search and typo tolerance
- [ ] Search result context display
- [ ] Multiple output format support (JSON, CSV)
- [ ] Parallel document processing
- [ ] Intelligent caching system
- [ ] Desktop app version of the CLI

---

## Built-in Commands

| Command | Description |
|---------|-------------|
| `search <needles> <haystack>` | Search for terms in a single document |
| `batch --directory <dir> --needles-file <file>` | Process multiple documents |
| `validate <needles> <haystack>` | Validate file compatibility |
| `info <file>` | Display file information |
| `--interactive` | Launch interactive search mode |
| `--tui` | Launch terminal user interface |
| `--gui` | Launch graphical user interface (planned) |
| `--help` | Show help information |
| `--version` | Display version information |

---

## Usage Examples

### Installation
```sh
# Clone and build
git clone https://github.com/Adel-Ayoub/docsearcher.git
cd docsearcher
cargo build --release

# Test the installation
cargo run -- --help
```

### Basic Search Operations
```bash
# Search a single PDF file
cargo run -- search contacts.csv document.pdf

# Search a DOCX file
cargo run -- search search_terms.csv report.docx

# Validate files before searching
cargo run -- validate contacts.csv document.pdf
```

### Batch Processing
```bash
# Process all PDFs in a directory
cargo run -- batch --directory ./documents --needles-file contacts.csv --pattern "*.pdf"

# Process all document types recursively
cargo run -- batch --directory ./documents --needles-file contacts.csv --recursive

# Custom file pattern matching
cargo run -- batch --directory ./documents --needles-file contacts.csv --pattern "*.docx"
```

### Interactive Modes
```bash
# Launch interactive CLI mode
cargo run -- --interactive

# Launch full TUI interface
cargo run -- --tui

# Launch GUI interface (planned)
cargo run -- --gui
```

### File Validation
```bash
# Check file compatibility
cargo run -- validate contacts.csv document.pdf

# Validate multiple files
cargo run -- validate contacts.csv document1.pdf document2.docx
```

### Advanced Usage
```bash
# Custom output format
cargo run -- batch --directory ./docs --needles-file terms.csv --format json

# Recursive directory search
cargo run -- batch --directory ./projects --needles-file keywords.csv --recursive --pattern "*.pdf"

# Search with specific patterns
cargo run -- batch --directory ./documents --needles-file search.csv --pattern "report_*.pdf"
```

---

## Search Term Format

Create a CSV file with your search terms:

```csv
term,category
algorithm,computer science
programming,software development
data structure,computer science
dynamic programming,algorithm design
graph theory,mathematics
```

## Supported File Types

| Format | Extension | Parser |
|--------|-----------|--------|
| PDF | `.pdf` | `pdf-extract` |
| DOCX | `.docx` | `docx` |

---




