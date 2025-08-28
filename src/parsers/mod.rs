pub mod docx;
pub mod pdf;

pub use docx::parse_from_path as parse_docx_from_path;
pub use pdf::parse_from_path as parse_pdf_from_path;
