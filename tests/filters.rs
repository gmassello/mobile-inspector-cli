use std::fs;

const ANDROID_FIXTURE: &str = "tests/fixtures/android_dump.xml";
const IOS_FIXTURE: &str = "tests/fixtures/ios_dump.xml";

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("no se pudo leer {path}: {e}"))
}

mod attrs {
    use super::*;
    use mobile_inspector_cli::cli::AttrFilters;
    use mobile_inspector_cli::filter::attrs::filter;

    #[test]
    fn android_filtro_por_resource_id_regex() {
        let xml = read(ANDROID_FIXTURE);
        let f = AttrFilters {
            id: Some("btn_.*".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert_eq!(nodes.len(), 2, "deberia matchear btn_continue y btn_cancel");
    }

    #[test]
    fn android_filtro_por_text_exacto() {
        let xml = read(ANDROID_FIXTURE);
        let f = AttrFilters {
            text: Some("^Continuar$".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(
            nodes[0].attrs.get("text").map(String::as_str),
            Some("Continuar")
        );
    }

    #[test]
    fn android_filtros_combinados_son_and() {
        let xml = read(ANDROID_FIXTURE);
        let f = AttrFilters {
            id: Some("btn_.*".into()),
            text: Some("Continuar".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn android_filtro_sin_match_devuelve_vacio() {
        let xml = read(ANDROID_FIXTURE);
        let f = AttrFilters {
            id: Some("no_existe".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn ios_filtro_id_machea_contra_name() {
        let xml = read(IOS_FIXTURE);
        let f = AttrFilters {
            id: Some("btn_.*".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn ios_filtro_text_machea_contra_label() {
        let xml = read(IOS_FIXTURE);
        let f = AttrFilters {
            text: Some("Cancelar".into()),
            ..Default::default()
        };
        let nodes = filter(&xml, &f).unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn regex_invalida_devuelve_error() {
        let xml = read(ANDROID_FIXTURE);
        let f = AttrFilters {
            id: Some("(.*".into()),
            ..Default::default()
        };
        assert!(filter(&xml, &f).is_err());
    }
}

mod xpath {
    use super::*;
    use mobile_inspector_cli::filter::xpath::filter;

    #[test]
    fn android_xpath_clickable() {
        let xml = read(ANDROID_FIXTURE);
        let nodes = filter(&xml, "//node[@clickable='true']").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn android_xpath_por_resource_id_exacto() {
        let xml = read(ANDROID_FIXTURE);
        let nodes = filter(&xml, "//node[@resource-id='cl.mach.app:id/btn_continue']").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn ios_xpath_por_name() {
        let xml = read(IOS_FIXTURE);
        let nodes = filter(&xml, "//*[@name='btn_continue']").unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn xpath_invalido_devuelve_error() {
        let xml = read(ANDROID_FIXTURE);
        assert!(filter(&xml, "//node[").is_err());
    }
}

mod output {
    use super::*;
    use mobile_inspector_cli::cli::OutputFormat;
    use mobile_inspector_cli::filter::FilterResult;
    use mobile_inspector_cli::filter::xpath::filter;
    use mobile_inspector_cli::model::UiNode;
    use mobile_inspector_cli::output::render;

    #[test]
    fn json_de_arbol_es_parseable() {
        let xml = read(ANDROID_FIXTURE);
        let tree = UiNode::parse_tree(&xml).unwrap();
        let out = render(&FilterResult::Tree(tree), OutputFormat::Json).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["type"], "tree");
    }

    #[test]
    fn json_de_lista_incluye_count() {
        let xml = read(ANDROID_FIXTURE);
        let nodes = filter(&xml, "//node[@clickable='true']").unwrap();
        let out = render(&FilterResult::Nodes(nodes), OutputFormat::Json).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["count"], 2);
        assert_eq!(v["type"], "list");
    }

    #[test]
    fn table_incluye_headers() {
        let xml = read(ANDROID_FIXTURE);
        let nodes = filter(&xml, "//node[@clickable='true']").unwrap();
        let out = render(&FilterResult::Nodes(nodes), OutputFormat::Table).unwrap();
        assert!(out.contains("id/name"));
        assert!(out.contains("bounds"));
    }
}
