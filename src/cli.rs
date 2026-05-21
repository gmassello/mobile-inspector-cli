use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "mobile-inspector",
    version,
    about = "Inspecciona el view hierarchy de Android (adb) o iOS (Appium)"
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Dump del view hierarchy actual (one-shot)
    Dump(DumpArgs),
    /// Sesion interactiva (REPL)
    Repl(ReplArgs),
    /// Gestion de configuracion
    Config(ConfigArgs),
}

#[derive(Debug, Args, Clone, Default)]
pub struct AttrFilters {
    /// Filtro regex contra resource-id (android) o name (ios)
    #[arg(long)]
    pub id: Option<String>,

    /// Filtro regex contra text (android) o label (ios)
    #[arg(long)]
    pub text: Option<String>,

    /// Filtro regex contra class (android) o type (ios)
    #[arg(long)]
    pub class: Option<String>,

    /// Filtro regex contra content-desc (android) o value (ios)
    #[arg(long = "content-desc")]
    pub content_desc: Option<String>,
}

impl AttrFilters {
    pub fn any(&self) -> bool {
        self.id.is_some()
            || self.text.is_some()
            || self.class.is_some()
            || self.content_desc.is_some()
    }
}

#[derive(Debug, Args, Clone)]
pub struct DumpArgs {
    #[arg(value_enum)]
    pub platform: Platform,

    #[command(flatten)]
    pub filters: AttrFilters,

    /// Expresion XPath. Si se pasa, ignora los filtros por atributos.
    #[arg(long)]
    pub xpath: Option<String>,

    /// Formato de salida
    #[arg(long, value_enum, default_value_t = OutputFormat::Xml)]
    pub format: OutputFormat,

    /// Serial del device adb (solo android). Si hay uno solo, opcional.
    #[arg(long)]
    pub serial: Option<String>,

    /// Session id de Appium (solo ios). Si hay una sola activa, opcional.
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Debug, Args)]
pub struct ReplArgs {
    #[arg(value_enum)]
    pub platform: Platform,
    #[arg(long)]
    pub serial: Option<String>,
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Imprime el valor de una clave (ej: appium.url)
    Get { key: String },
    /// Asigna un valor a una clave
    Set { key: String, value: String },
    /// Imprime la ruta del archivo de configuracion
    Path,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Xml,
    Json,
    Table,
}
