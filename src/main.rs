use anyhow::Result;
use clap::Parser;
use mobile_inspector_cli::cli::{self, Cli, Command};
use mobile_inspector_cli::{config, filter, output, platform, repl};

fn main() -> Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbose);

    match args.command {
        Command::Dump(d) => run_dump(d),
        Command::Repl(r) => repl::run(r),
        Command::Config(c) => config::handle_config(c),
    }
}

fn init_tracing(verbose: bool) {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };
    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

fn run_dump(args: cli::DumpArgs) -> Result<()> {
    use cli::Platform as P;
    let platform: Box<dyn platform::Platform> = match args.platform {
        P::Android => Box::new(platform::android::AdbPlatform::new(args.serial.clone())),
        P::Ios => {
            let cfg = config::Config::load()?;
            Box::new(platform::ios::AppiumPlatform::new(
                cfg.appium.url,
                args.session.clone(),
            ))
        }
    };

    let xml = platform.dump_xml()?;
    let filtered = filter::apply_filters(&xml, &args)?;
    let rendered = output::render(&filtered, args.format)?;
    print!("{}", rendered);
    Ok(())
}
