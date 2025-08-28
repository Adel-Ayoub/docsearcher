use std::collections::HashSet;

/// Represents a search term with its associated metadata
pub type Needle<'a> = (&'a str, &'a str);

/// Represents a search result with the found term and metadata
pub type SearchResult = (String, String);

/// Supported document file types
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    /// Microsoft Word document (.docx)
    Docx,
    /// Portable Document Format (.pdf)
    Pdf,
}

impl FileType {
    /// Get the file extension for this file type
    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Docx => ".docx",
            FileType::Pdf => ".pdf",
        }
    }
    
    /// Get the MIME type for this file type
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileType::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            FileType::Pdf => "application/pdf",
        }
    }
}

/// Collection of search results
pub type SearchResults = HashSet<SearchResult>;
