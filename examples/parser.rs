#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{self, prelude::*};
use std::path::PathBuf;

use failure::{format_err, Error};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "parser",
    about = "Parse a PDL format file and generate JSON or Markdown file"
)]
struct Opt {
    /// Dump to the PDL format
    #[structopt(short, long)]
    pdl: bool,

    /// Dump to the JSON format
    #[structopt(short, long)]
    json: bool,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// file in PDL format
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

impl Opt {
    fn dump(&self, proto: &pdl::Protocol) -> Result<(), Error> {
        match self.output {
            Some(ref filename) if filename.to_str() != Some("-") => {
                let mut f = File::create(filename)?;

                self.dump_to(&mut f, proto)
            }
            _ => {
                let stdout = io::stdout();
                let mut h = stdout.lock();

                self.dump_to(&mut h, proto)
            }
        }
    }

    fn dump_to<W: Write>(&self, w: &mut W, proto: &pdl::Protocol) -> Result<(), Error> {
        if self.json {
            write!(w, "{}", proto.to_json_pretty()?)?;
        } else if self.pdl {
            write!(w, "{}", proto.to_string())?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let opt = Opt::from_args();
    debug!("opt: {:#?}", opt);

    let mut f = File::open(&opt.file)?;
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

    opt.dump(&protocol)
}
