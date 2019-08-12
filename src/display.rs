use std::fmt;

use crate::*;

fn indented<D: fmt::Display>(data: D) -> indented::Indented<D, indented::Space2> {
    indented::indented_with(data, indented::Space2)
}

impl fmt::Display for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for comment in &self.0 {
            writeln!(f, "# {}", comment)?;
        }

        Ok(())
    }
}

impl fmt::Display for Protocol<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }
        writeln!(f, "{}", self.version)?;

        for domain in &self.domains {
            write!(f, "{}", domain)?;
        }

        Ok(())
    }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "version\n  major {}\n  minor {}", self.major, self.minor)
    }
}

impl fmt::Display for Domain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }

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
            writeln!(f, "{}", indented(format!("depends on {}", depends)))?;
        }

        writeln!(f)?;

        for ty in &self.types {
            writeln!(f, "{}", indented(ty))?;
        }
        for cmd in &self.commands {
            writeln!(f, "{}", indented(cmd))?;
        }
        for evt in &self.events {
            writeln!(f, "{}", indented(evt))?;
        }

        Ok(())
    }
}

impl fmt::Display for TypeDef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }

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
            write!(f, "{}", indented(item))?;
        }

        Ok(())
    }
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Enum(_) => f.write_str("enum"),
            Type::Integer => f.write_str("integer"),
            Type::Number => f.write_str("number"),
            Type::Boolean => f.write_str("boolean"),
            Type::String => f.write_str("string"),
            Type::Object => f.write_str("object"),
            Type::Any => f.write_str("any"),
            Type::Binary => f.write_str("binary"),
            Type::ArrayOf(ty) => write!(f, "array of {}", ty),
            Type::Ref(id) => f.write_str(id),
        }
    }
}

impl fmt::Display for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Item::Enum(variants) => write!(f, "{}", Enum(Some("enum"), variants)),
            Item::Properties(props) => write!(f, "{}", Params("properties", props.as_slice())),
        }
    }
}

struct Enum<'a>(Option<&'a str>, &'a [Variant<'a>]);

impl fmt::Display for Enum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.0 {
            writeln!(f, "{}", name)?;
        }

        for variant in self.1 {
            writeln!(f, "{}", indented(variant))?;
        }

        Ok(())
    }
}

impl fmt::Display for Variant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }
        f.write_str(self.name)
    }
}

struct Params<'a>(&'a str, &'a [Param<'a>]);

impl fmt::Display for Params<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.0)?;

        for param in self.1 {
            write!(f, "{}", indented(param))?;
        }

        Ok(())
    }
}

impl fmt::Display for Param<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }

        writeln!(
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
        )?;

        if let Type::Enum(ref variants) = self.ty {
            write!(f, "{}", Enum(None, variants))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Command<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }

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
            write!(f, "{}", indented(redirect))?;
        }

        if !self.parameters.is_empty() {
            write!(f, "{}", indented(Params("parameters", &self.parameters)))?;
        }
        if !self.returns.is_empty() {
            write!(f, "{}", indented(Params("returns", &self.returns)))?;
        }

        Ok(())
    }
}

impl fmt::Display for Event<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }

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

        if !self.parameters.is_empty() {
            write!(f, "{}", indented(Params("parameters", &self.parameters)))?;
        }

        Ok(())
    }
}

impl fmt::Display for Redirect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{}", self.description)?;
        }
        writeln!(f, "redirect {}", self.to)
    }
}
