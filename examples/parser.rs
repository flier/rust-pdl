#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use failure::{format_err, Error};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "parser",
    about = "Parse a PDL format file and generate JSON or Markdown file"
)]
struct Opt {
    /// file in PDL format
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let opt = Opt::from_args();
    debug!("opt: {:#?}", opt);

    let mut f = File::open(opt.file)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let (rest, protocol) = pdl::parse(&s).map_err(|err| {
        format_err!(
            "fail to parse PDL file, {}",
            match err {
                nom::Err::Incomplete(_) => format!("incomplete input"),
                nom::Err::Error((_, err)) | nom::Err::Failure((_, err)) => {
                    format!("{}", err.description())
                }
            }
        )
    })?;

    if !rest.is_empty() {
        warn!("unexpected: {}", &rest[..1000]);
    }
    trace!("protocol: {:#?}", protocol);

    Ok(())
}
