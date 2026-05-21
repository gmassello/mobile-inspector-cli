use crate::cli::DumpArgs;
use crate::error::Result;
use crate::model::UiNode;

pub mod attrs;
pub mod xpath;

pub enum FilterResult {
    Tree(UiNode),
    Nodes(Vec<UiNode>),
}

pub fn apply_filters(xml: &str, args: &DumpArgs) -> Result<FilterResult> {
    if let Some(expr) = args.xpath.as_deref() {
        return Ok(FilterResult::Nodes(xpath::filter(xml, expr)?));
    }
    if args.filters.any() {
        return Ok(FilterResult::Nodes(attrs::filter(xml, &args.filters)?));
    }
    Ok(FilterResult::Tree(UiNode::parse_tree(xml)?))
}
