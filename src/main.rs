use anyhow::{anyhow, Context, Result};
use std::{env, fs, path::PathBuf};

use wit_parser::{Resolve, UnresolvedPackage};
use waldo_parser::Document;

// waldo-parser [WIT] [DOCUMENT]

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let wit_arg = args
        .next()
        .map(PathBuf::from)
        .ok_or(anyhow!("missing WIT argument"))?;

    let mut resolve = Resolve::default();
    if wit_arg.is_dir() {
        let pkg = if wit_arg.is_dir() {
            resolve.push_dir(&wit_arg)?.0
        } else {
            resolve.push(UnresolvedPackage::parse_file(&wit_arg)?)?
        };   
    }

    let wld_arg = args
        .next()
        .map(PathBuf::from)
        .ok_or(anyhow!("missing path argument"))?;

    let input = fs::read_to_string(&wld_arg)
            .with_context(|| anyhow!("failed to read document at {wld_arg:?}"))?;

    let _document = Document::parse(&resolve, &wld_arg, &input)?;

    Ok(())
}