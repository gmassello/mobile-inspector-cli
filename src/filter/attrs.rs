use crate::attr;
use crate::cli::AttrFilters;
use crate::error::{InspectorError, Result};
use crate::model::UiNode;
use regex::Regex;

pub fn filter(xml: &str, filters: &AttrFilters) -> Result<Vec<UiNode>> {
    let tree = UiNode::parse_tree(xml)?;
    filter_tree(&tree, filters)
}

pub fn filter_tree(tree: &UiNode, filters: &AttrFilters) -> Result<Vec<UiNode>> {
    let matchers = Matchers::build(filters)?;
    Ok(tree
        .iter_descendants()
        .filter(|n| matches_node(n, &matchers))
        .map(UiNode::flatten_attrs)
        .collect())
}

struct Matchers {
    id: Option<Regex>,
    text: Option<Regex>,
    class: Option<Regex>,
    content_desc: Option<Regex>,
}

impl Matchers {
    fn build(f: &AttrFilters) -> Result<Self> {
        Ok(Self {
            id: compile(&f.id)?,
            text: compile(&f.text)?,
            class: compile(&f.class)?,
            content_desc: compile(&f.content_desc)?,
        })
    }
}

fn compile(p: &Option<String>) -> Result<Option<Regex>> {
    match p {
        Some(s) => Regex::new(s)
            .map(Some)
            .map_err(|e| InspectorError::Filter(format!("regex invalida '{s}': {e}"))),
        None => Ok(None),
    }
}

fn matches_node(node: &UiNode, m: &Matchers) -> bool {
    matches_any(&m.id, node, attr::ID)
        && matches_any(&m.text, node, attr::TEXT)
        && matches_any(&m.class, node, attr::CLASS)
        && matches_any(&m.content_desc, node, attr::CONTENT_DESC)
}

fn matches_any(re: &Option<Regex>, node: &UiNode, keys: &[&str]) -> bool {
    match re {
        None => true,
        Some(r) => keys
            .iter()
            .filter_map(|k| node.attrs.get(*k))
            .any(|v| r.is_match(v)),
    }
}
