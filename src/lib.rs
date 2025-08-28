pub mod parsers;
pub mod types;
pub mod utils;
pub mod cmd;

pub use parsers::{parse_docx_from_path, parse_pdf_from_path};
pub use types::{FileType, SearchResult};
pub use utils::{parse_filetype, read_needles_from_file, read_needles_from_mem};
