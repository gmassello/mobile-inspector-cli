use crate::error::{InspectorError, Result};
use crate::model::UiNode;
use std::collections::BTreeMap;
use sxd_document::parser;
use sxd_xpath::{Context, Factory, Value};

pub fn filter(xml: &str, expr: &str) -> Result<Vec<UiNode>> {
    let package = parser::parse(xml).map_err(|e| InspectorError::XmlParse(e.to_string()))?;
    let document = package.as_document();

    let factory = Factory::new();
    let xpath = factory
        .build(expr)
        .map_err(|e| InspectorError::XPath(e.to_string()))?
        .ok_or_else(|| InspectorError::XPath("expresion xpath vacia".into()))?;
    let context = Context::new();

    let value = xpath
        .evaluate(&context, document.root())
        .map_err(|e| InspectorError::XPath(e.to_string()))?;

    let mut out = Vec::new();
    if let Value::Nodeset(ns) = value {
        for node in ns.document_order() {
            if let Some(el) = node.element() {
                let mut attrs = BTreeMap::new();
                for a in el.attributes() {
                    attrs.insert(a.name().local_part().to_string(), a.value().to_string());
                }
                out.push(UiNode {
                    tag: el.name().local_part().to_string(),
                    attrs,
                    children: Vec::new(),
                });
            }
        }
    }
    Ok(out)
}
