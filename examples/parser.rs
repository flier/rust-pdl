#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{self, prelude::*};
use std::mem;
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

    /// Dump API to Markdown documentation
    #[structopt(short, long)]
    markdown: bool,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Open the generated file using the program configured on the system
    #[structopt(long)]
    open: bool,

    /// file in PDL format
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

impl Opt {
    fn dump(&self, proto: &pdl::Protocol) -> Result<(), Error> {
        match self.output {
            Some(ref filename) if filename.to_str() != Some("-") => {
                let mut f = File::create(filename)?;

                self.dump_to(&mut f, proto)?;

                mem::drop(f);

                if self.open {
                    open::that(filename)?;
                }

                Ok(())
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
        } else if self.markdown {
            markdown::render(w, &self.file.file_name().unwrap().to_string_lossy(), proto)?;
        }

        Ok(())
    }
}

mod markdown {
    use std::io;

    use failure::Error;

    fn write_params<W: io::Write>(w: &mut W, name: &str, params: &[pdl::Param]) -> io::Result<()> {
        if !params.is_empty() {
            writeln!(
                w,
                r"_{}_

| Name | Type | Description |
| ----:|:---- | :---------- |",
                name.to_uppercase(),
            )?;

            for param in params {
                writeln!(
                    w,
                    "| `{}`{} | **{}** | {}{}{} |",
                    param.name,
                    if param.optional {
                        " <br/> _optional_ "
                    } else {
                        ""
                    },
                    param.ty,
                    if param.description.is_empty() {
                        "".to_string()
                    } else {
                        format!("_{}_", param.description.join(" "))
                    },
                    if param.experimental {
                        " **_EXPERIMENTAL_**"
                    } else {
                        ""
                    },
                    if param.deprecated {
                        " **_DEPRECATED_**"
                    } else {
                        ""
                    }
                )?;
            }

            writeln!(w, "")
        } else {
            Ok(())
        }
    }

    pub fn render<W: io::Write>(w: &mut W, name: &str, proto: &pdl::Protocol) -> Result<(), Error> {
        writeln!(
            w,
            "# {}\n\nversion: {}.{}\n",
            name, proto.version.major, proto.version.minor
        )?;

        for domain in &proto.domains {
            writeln!(w, "## {} Domain\n", domain.name)?;
            writeln!(w, "> {}", domain.description.join(" "))?;

            writeln!(
                w,
                ">{}{}\n",
                if domain.experimental {
                    " **_EXPERIMENTAL_**"
                } else {
                    ""
                },
                if domain.deprecated {
                    " **_DEPRECATED_**"
                } else {
                    ""
                }
            )?;

            if !domain.commands.is_empty() {
                writeln!(w, "### Methods\n")?;

                for cmd in &domain.commands {
                    writeln!(w, "_{}._**{}**\n", domain.name, cmd.name)?;
                    writeln!(w, "> {}", cmd.description.join(" "))?;
                    writeln!(
                        w,
                        ">{}{}\n",
                        if cmd.experimental {
                            " **_EXPERIMENTAL_**"
                        } else {
                            ""
                        },
                        if cmd.deprecated {
                            " **_DEPRECATED_**"
                        } else {
                            ""
                        }
                    )?;

                    write_params(w, "Parameters", &cmd.parameters)?;
                    write_params(w, "Return Object", &cmd.returns)?;

                    writeln!(w, "---\n")?;
                }
            }

            if !domain.types.is_empty() {
                writeln!(w, "### Types\n")?;

                for ty in &domain.types {
                    writeln!(w, "_{}._**{}**\n", domain.name, ty.id)?;
                    writeln!(w, "> {}", ty.description.join(" "))?;
                    writeln!(
                        w,
                        ">{}{}\n",
                        if ty.experimental {
                            " **_EXPERIMENTAL_**"
                        } else {
                            ""
                        },
                        if ty.deprecated {
                            " **_DEPRECATED_**"
                        } else {
                            ""
                        }
                    )?;

                    writeln!(w, "\nType: **{}**\n", ty.extends)?;

                    match ty.item {
                        Some(pdl::Item::Enum(ref variants)) => {
                            writeln!(
                                w,
                                r"_ALLOWED VALUES_

| Name | Description |
| ----:|:----------- |"
                            )?;

                            for variant in variants {
                                writeln!(
                                    w,
                                    "| `{}` | {} |",
                                    variant.name,
                                    if variant.description.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!("_{}_", variant.description.join(" "))
                                    },
                                )?;
                            }
                        }
                        Some(pdl::Item::Properties(ref props)) => {
                            write_params(w, "Properties", props)?;
                        }
                        None => {}
                    }

                    writeln!(w, "---\n")?;
                }

                writeln!(w, "")?;
            }

            if !domain.events.is_empty() {
                writeln!(w, "### Events\n")?;

                for evt in &domain.events {
                    writeln!(w, "_{}._**{}**\n", domain.name, evt.name)?;
                    writeln!(w, "> {}", evt.description.join(" "))?;
                    writeln!(
                        w,
                        ">{}{}\n",
                        if evt.experimental {
                            " **_EXPERIMENTAL_**"
                        } else {
                            ""
                        },
                        if evt.deprecated {
                            " **_DEPRECATED_**"
                        } else {
                            ""
                        }
                    )?;

                    write_params(w, "Parameters", &evt.parameters)?;
                }

                writeln!(w, "---\n")?;
            }
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
