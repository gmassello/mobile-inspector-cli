use crate::error::{InspectorError, Result};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct UiNode {
    pub tag: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn parse_tree(xml: &str) -> Result<UiNode> {
        let doc =
            roxmltree::Document::parse(xml).map_err(|e| InspectorError::XmlParse(e.to_string()))?;
        Ok(Self::from_node(doc.root_element()))
    }

    fn from_node(node: roxmltree::Node) -> UiNode {
        let mut attrs = BTreeMap::new();
        for a in node.attributes() {
            attrs.insert(a.name().to_string(), a.value().to_string());
        }
        let children = node
            .children()
            .filter(|c| c.is_element())
            .map(Self::from_node)
            .collect();
        UiNode {
            tag: node.tag_name().name().to_string(),
            attrs,
            children,
        }
    }

    pub fn iter_descendants(&self) -> impl Iterator<Item = &UiNode> {
        let mut stack: Vec<&UiNode> = vec![self];
        std::iter::from_fn(move || {
            let n = stack.pop()?;
            for c in n.children.iter().rev() {
                stack.push(c);
            }
            Some(n)
        })
    }

    pub fn flatten_attrs(&self) -> UiNode {
        UiNode {
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
            children: Vec::new(),
        }
    }
}
