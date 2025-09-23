use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

use site::{Config, Result, Site};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Build {
        #[structopt(long = "root", default_value = ".")]
        root: String,
        #[structopt(long = "config")]
        config: Option<String>,
        #[structopt(long = "out")]
        out: String,
        #[structopt(long = "article-regex")]
        article_regex: Option<String>,
    },
}

fn main() -> Result<()> {
    let opt = Cli::parse();
    env_logger::init();
    match opt.cmd {
        Command::Build {
            config,
            root,
            out,
            article_regex,
        } => {
            let root = PathBuf::from(root);
            let config = {
                let mut default_config = Config::read(root.join("config.toml"))?;
                if let Some(config) = config.as_ref() {
                    default_config.extend(&mut Config::read(config)?);
                }
                default_config
            };
            let app = Site::new(
                config,
                root,
                PathBuf::from(out),
                article_regex.map(|regex| Regex::new(&regex).expect("invalid regex")),
            );
            app.build()
        }
    }
}
