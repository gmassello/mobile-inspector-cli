use crate::error::Result;
use crate::filter::FilterResult;
use serde::Serialize;

#[derive(Serialize)]
struct TreeOutput<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    root: &'a crate::model::UiNode,
}

#[derive(Serialize)]
struct ListOutput<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    count: usize,
    nodes: &'a [crate::model::UiNode],
}

pub fn render(result: &FilterResult) -> Result<String> {
    let s = match result {
        FilterResult::Tree(root) => {
            serde_json::to_string_pretty(&TreeOutput { kind: "tree", root })?
        }
        FilterResult::Nodes(nodes) => serde_json::to_string_pretty(&ListOutput {
            kind: "list",
            count: nodes.len(),
            nodes,
        })?,
    };
    Ok(s)
}
