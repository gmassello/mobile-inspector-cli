use crate::cli::{AttrFilters, OutputFormat, Platform, ReplArgs};
use crate::config::Config;
use crate::error::Result;
use crate::filter::FilterResult;
use crate::model::UiNode;
use crate::output;
use crate::platform::{self, Platform as PlatformTrait};
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Debug, Parser)]
#[command(
    no_binary_name = true,
    name = "find",
    about = "Filtra el dump cacheado por atributos"
)]
struct FindCmd {
    #[command(flatten)]
    filters: AttrFilters,
}

#[derive(Default)]
struct Cache {
    xml: Option<String>,
    tree: Option<UiNode>,
}

impl Cache {
    fn ensure(&mut self, platform: &dyn PlatformTrait) -> Result<()> {
        if self.xml.is_none() {
            self.refresh(platform)?;
        }
        Ok(())
    }

    fn refresh(&mut self, platform: &dyn PlatformTrait) -> Result<()> {
        let xml = platform.dump_xml()?;
        let tree = UiNode::parse_tree(&xml)?;
        self.xml = Some(xml);
        self.tree = Some(tree);
        Ok(())
    }

    fn tree(&self) -> &UiNode {
        self.tree.as_ref().expect("ensure() should be called first")
    }
}

pub fn run(args: ReplArgs) -> anyhow::Result<()> {
    let platform = build_platform(&args)?;
    let mut cache = Cache::default();
    let mut rl = DefaultEditor::new()?;
    let prompt = format!("[{}] > ", platform.name());

    println!(
        "mobile-inspector REPL ({}). 'help' para comandos, 'exit' para salir.",
        platform.name()
    );

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line).ok();
                if line == "exit" || line == "quit" {
                    break;
                }
                if let Err(e) = handle_line(line, &*platform, &mut cache) {
                    eprintln!("error: {e}");
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        }
    }
    Ok(())
}

fn build_platform(args: &ReplArgs) -> anyhow::Result<Box<dyn PlatformTrait>> {
    let p: Box<dyn PlatformTrait> = match args.platform {
        Platform::Android => Box::new(platform::android::AdbPlatform::new(args.serial.clone())),
        Platform::Ios => {
            let cfg = Config::load()?;
            Box::new(platform::ios::AppiumPlatform::new(
                cfg.appium.url,
                args.session.clone(),
            ))
        }
    };
    Ok(p)
}

fn handle_line(line: &str, platform: &dyn PlatformTrait, cache: &mut Cache) -> anyhow::Result<()> {
    let mut parts = line.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let rest: Vec<&str> = parts.collect();

    match cmd {
        "help" | "?" => print_help(),
        "refresh" => {
            cache.refresh(platform)?;
            println!("refreshed");
        }
        "dump" => {
            cache.ensure(platform)?;
            let r = FilterResult::Tree(cache.tree().clone());
            println!("{}", output::render(&r, OutputFormat::Xml)?);
        }
        "find" => {
            cache.ensure(platform)?;
            let argv = std::iter::once("find").chain(rest.iter().copied());
            match FindCmd::try_parse_from(argv) {
                Ok(cmd) => {
                    let nodes = crate::filter::attrs::filter_tree(cache.tree(), &cmd.filters)?;
                    let r = FilterResult::Nodes(nodes);
                    println!("{}", output::render(&r, OutputFormat::Table)?);
                }
                Err(e) => print!("{e}"),
            }
        }
        "xpath" => {
            if rest.is_empty() {
                println!("uso: xpath <expr>");
            } else {
                cache.ensure(platform)?;
                let expr = rest.join(" ");
                let xml = cache.xml.as_deref().expect("xml cached");
                let nodes = crate::filter::xpath::filter(xml, &expr)?;
                let r = FilterResult::Nodes(nodes);
                println!("{}", output::render(&r, OutputFormat::Table)?);
            }
        }
        other => println!("comando desconocido: {other}. tipea 'help'"),
    }
    Ok(())
}

fn print_help() {
    println!("comandos:");
    println!("  dump                       imprime view hierarchy completo (usa cache)");
    println!("  refresh                    fuerza nuevo dump del device");
    println!("  find --id|--text|--class|--content-desc <regex>");
    println!("  xpath <expr>               aplica xpath sobre el dump cacheado");
    println!("  help | exit");
}
