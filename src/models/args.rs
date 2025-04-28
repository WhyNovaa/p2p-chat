use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    pub dir: Option<PathBuf>,
}
