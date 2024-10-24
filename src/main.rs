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
        #[structopt(long = "root-dir", default_value = ".")]
        root_dir: String,
        #[structopt(long = "config")]
        config: Option<String>,
        #[structopt(long = "out-dir")]
        out_dir: String,
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
            root_dir,
            out_dir,
            article_regex,
        } => {
            let root_dir = PathBuf::from(root_dir);
            let config = {
                let mut default_config = Config::read(root_dir.join("config.toml"))?;
                if let Some(config) = config.as_ref() {
                    default_config.extend(&mut Config::read(config)?);
                }
                default_config
            };
            let app = Site::new(
                config,
                root_dir,
                PathBuf::from(out_dir),
                article_regex.map(|regex| Regex::new(&regex).expect("invalid regex")),
            );
            app.build()
        }
    }
}
