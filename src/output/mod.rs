use crate::cli::OutputFormat;
use crate::error::Result;
use crate::filter::FilterResult;

pub mod json;
pub mod table;
pub mod xml;

pub fn render(result: &FilterResult, fmt: OutputFormat) -> Result<String> {
    match fmt {
        OutputFormat::Xml => xml::render(result),
        OutputFormat::Json => json::render(result),
        OutputFormat::Table => table::render(result),
    }
}
