use crate::error::Result;

pub mod android;
pub mod ios;

pub trait Platform {
    fn dump_xml(&self) -> Result<String>;
    fn name(&self) -> &'static str;
}
