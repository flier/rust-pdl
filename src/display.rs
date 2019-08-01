use std::fmt::{self, Write};

use crate::*;

struct Ident<W>(W);

impl<W: fmt::Write> fmt::Write for Ident<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(&s.replace("\n", "\n  "))
    }
}

fn write_description<W: fmt::Write>(f: &mut W, description: &[&str]) -> fmt::Result {
    for comment in description {
        writeln!(f, "# {}", comment)?;
    }

    Ok(())
}

fn write_params<W: fmt::Write>(f: &mut W, name: &str, params: &[Param<'_>]) -> fmt::Result {
    if !params.is_empty() {
        let f = &mut Ident(f);

        writeln!(f, "{}", name)?;

        for param in params {
            write_description(f, &param.description)?;
            writeln!(f, "{}", param)?;
        }
    }

    Ok(())
}

impl fmt::Display for Protocol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_description(f, &self.description)?;
        writeln!(
            f,
            r#"
version
  major {}
  minor {}
"#,
            self.version.0, self.version.1
        )?;

        for domain in &self.domains {
            write!(f, "{}", domain)?;
        }

        Ok(())
    }
}

impl fmt::Display for Domain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_description(f, &self.description)?;

        let f = &mut Ident(f);

        writeln!(
            f,
            "{}{}domain {}",
            if self.experimental {
                "experimental "
            } else {
                ""
            },
            if self.deprecated { "deprecated " } else { "" },
            self.name,
        )?;

        for depends in &self.dependencies {
            writeln!(f, "depends on {}", depends)?;
        }

        writeln!(f, "")?;

        for ty in &self.types {
            writeln!(f, "{}", ty)?;
        }
        for cmd in &self.commands {
            writeln!(f, "{}", cmd)?;
        }
        for evt in &self.events {
            writeln!(f, "{}", evt)?;
        }

        Ok(())
    }
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_description(f, &self.description)?;

        let f = &mut Ident(f);

        writeln!(
            f,
            "{}{}type {} extends {}",
            if self.experimental {
                "experimental "
            } else {
                ""
            },
            if self.deprecated { "deprecated " } else { "" },
            self.id,
            self.extends
        )?;

        if let Some(ref item) = self.item {
            write!(f, "{}", item)?;
        }

        Ok(())
    }
}

impl fmt::Display for Ty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ty::Enum(_) => f.write_str("enum"),
            Ty::Integer => f.write_str("integer"),
            Ty::Number => f.write_str("number"),
            Ty::Boolean => f.write_str("boolean"),
            Ty::String => f.write_str("string"),
            Ty::Object => f.write_str("object"),
            Ty::Any => f.write_str("any"),
            Ty::ArrayOf(ty) => write!(f, "array of {}", ty),
            Ty::Ref(id) => f.write_str(id),
        }
    }
}

impl fmt::Display for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f = &mut Ident(f);

        match self {
            Item::Enum(variants) => {
                writeln!(f, "enum")?;

                for variant in variants {
                    write_description(f, &variant.description)?;
                    writeln!(f, "{}", variant.name)?;
                }
            }
            Item::Properties(props) => {
                writeln!(f, "properties")?;

                for prop in props {
                    write_description(f, &prop.description)?;
                    writeln!(f, "{}", prop)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Variant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl fmt::Display for Param<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{} {}",
            if self.experimental {
                "experimental "
            } else {
                ""
            },
            if self.deprecated { "deprecated " } else { "" },
            if self.optional { "optional " } else { "" },
            self.ty,
            self.name
        )
    }
}

impl fmt::Display for Command<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_description(f, &self.description)?;

        let f = &mut Ident(f);

        writeln!(
            f,
            "{}{}command {}",
            if self.experimental {
                "experimental "
            } else {
                ""
            },
            if self.deprecated { "deprecated " } else { "" },
            self.name
        )?;

        if let Some(ref redirect) = self.redirect {
            writeln!(f, "{}", redirect)?;
        }

        write_params(f, "parameters", &self.parameters)?;
        write_params(f, "returns", &self.returns)?;

        Ok(())
    }
}

impl fmt::Display for Event<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_description(f, &self.description)?;

        let f = &mut Ident(f);

        writeln!(
            f,
            "{}{}event {}",
            if self.experimental {
                "experimental "
            } else {
                ""
            },
            if self.deprecated { "deprecated " } else { "" },
            self.name
        )?;

        write_params(f, "parameters", &self.parameters)
    }
}

impl fmt::Display for Redirect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "redirect {}", self.to)
    }
}
