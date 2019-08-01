#[cfg(feature = "to_json")]
use serde::Serialize;

#[cfg(feature = "parse")]
mod parse;
#[cfg(feature = "parse")]
pub use parse::parse;

#[cfg(feature = "to_string")]
mod display;

pub type Description<'a> = Vec<&'a str>;

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Protocol<'a> {
    pub description: Description<'a>,
    pub version: (usize, usize),
    pub domains: Vec<Domain<'a>>,
}

impl Protocol<'_> {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Domain<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub dependencies: Vec<&'a str>,
    pub types: Vec<Type<'a>>,
    pub commands: Vec<Command<'a>>,
    pub events: Vec<Event<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Type<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub optional: bool,
    pub id: &'a str,
    pub extends: Ty<'a>,
    pub item: Option<Item<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub enum Ty<'a> {
    Enum(Vec<Variant<'a>>),
    Integer,
    Number,
    Boolean,
    String,
    Object,
    Any,
    ArrayOf(Box<Ty<'a>>),
    Ref(&'a str),
}

impl Ty<'_> {
    pub fn new(ty: &str, is_array: bool) -> Ty {
        if is_array {
            Ty::ArrayOf(Box::new(Ty::new(ty, false)))
        } else {
            match ty {
                "enum" => Ty::Enum(vec![]),
                "integer" => Ty::Integer,
                "number" => Ty::Number,
                "boolean" => Ty::Boolean,
                "string" => Ty::String,
                "object" => Ty::Object,
                "any" => Ty::Any,
                _ => Ty::Ref(ty),
            }
        }
    }
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub enum Item<'a> {
    Enum(Vec<Variant<'a>>),
    Properties(Vec<Param<'a>>),
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Variant<'a> {
    pub description: Description<'a>,
    pub name: &'a str,
}

impl<'a> Variant<'a> {
    pub fn new(name: &str) -> Variant {
        Variant {
            description: vec![],
            name,
        }
    }
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Param<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub optional: bool,
    pub ty: Ty<'a>,
    pub name: &'a str,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub redirect: Option<Redirect<'a>>,
    pub parameters: Vec<Param<'a>>,
    pub returns: Vec<Param<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Event<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub parameters: Vec<Param<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Debug, PartialEq)]
pub struct Redirect<'a> {
    pub description: Description<'a>,
    pub to: &'a str,
}
