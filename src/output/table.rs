use crate::attr;
use crate::error::Result;
use crate::filter::FilterResult;
use crate::model::UiNode;
use std::fmt::Write;

pub fn render(result: &FilterResult) -> Result<String> {
    let nodes: Vec<&UiNode> = match result {
        FilterResult::Tree(root) => root.iter_descendants().collect(),
        FilterResult::Nodes(ns) => ns.iter().collect(),
    };

    let mut buf = String::new();
    writeln!(
        &mut buf,
        "{:<45} {:<30} {:<35} {:<25}",
        "id/name", "class/type", "text/label", "bounds"
    )
    .ok();
    writeln!(&mut buf, "{}", "-".repeat(140)).ok();
    for n in nodes {
        let id = pick(n, attr::ID);
        let cls = pick(n, attr::CLASS);
        let text = pick(n, attr::TEXT);
        let bounds = pick(n, &[attr::BOUNDS]);
        writeln!(
            &mut buf,
            "{:<45.45} {:<30.30} {:<35.35} {:<25.25}",
            id, cls, text, bounds
        )
        .ok();
    }
    Ok(buf)
}

fn pick<'a>(node: &'a UiNode, keys: &[&str]) -> &'a str {
    keys.iter()
        .filter_map(|k| node.attrs.get(*k))
        .find(|v| !v.is_empty())
        .map(String::as_str)
        .unwrap_or("")
}
