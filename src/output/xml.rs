use crate::error::Result;
use crate::filter::FilterResult;
use crate::model::UiNode;

pub fn render(result: &FilterResult) -> Result<String> {
    let mut buf = String::new();
    match result {
        FilterResult::Tree(root) => write_node(root, 0, &mut buf),
        FilterResult::Nodes(nodes) => {
            buf.push_str("<results>\n");
            for n in nodes {
                write_node(n, 1, &mut buf);
            }
            buf.push_str("</results>\n");
        }
    }
    Ok(buf)
}

fn write_node(node: &UiNode, depth: usize, buf: &mut String) {
    let pad = "  ".repeat(depth);
    buf.push_str(&pad);
    buf.push('<');
    buf.push_str(&node.tag);
    for (k, v) in &node.attrs {
        buf.push(' ');
        buf.push_str(k);
        buf.push_str("=\"");
        write_escaped(v, buf);
        buf.push('"');
    }
    if node.children.is_empty() {
        buf.push_str("/>\n");
    } else {
        buf.push_str(">\n");
        for c in &node.children {
            write_node(c, depth + 1, buf);
        }
        buf.push_str(&pad);
        buf.push_str("</");
        buf.push_str(&node.tag);
        buf.push_str(">\n");
    }
}

fn write_escaped(s: &str, buf: &mut String) {
    for c in s.chars() {
        match c {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            _ => buf.push(c),
        }
    }
}
