use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "parse")] {
        #[macro_use]
        extern crate log;

        mod parse;

        pub use parse::parse;
    }
}

#[cfg(feature = "display")]
mod display;

cfg_if! {
    if #[cfg(feature = "to_json")] {
        use serde::Serialize;

        mod ser;
    }
}

pub type Description<'a> = Vec<&'a str>;

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Protocol<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing))]
    pub description: Description<'a>,
    pub version: Version,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub domains: Vec<Domain<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    #[cfg_attr(feature = "to_json", serde(serialize_with = "ser::serialize_usize"))]
    pub major: usize,
    #[cfg_attr(feature = "to_json", serde(serialize_with = "ser::serialize_usize"))]
    pub minor: usize,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domain<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
    pub description: Description<'a>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub experimental: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub deprecated: bool,
    pub domain: &'a str,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub dependencies: Vec<&'a str>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub types: Vec<TypeDef<'a>>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub commands: Vec<Command<'a>>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub events: Vec<Event<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDef<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
    pub description: Description<'a>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub experimental: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub deprecated: bool,
    pub id: &'a str,
    #[cfg_attr(feature = "to_json", serde(flatten))]
    pub extends: Type<'a>,
    #[cfg_attr(feature = "to_json", serde(flatten))]
    pub item: Option<Item<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type<'a> {
    Integer,
    Number,
    Boolean,
    String,
    Object,
    Any,
    Enum(Vec<Variant<'a>>),
    ArrayOf(Box<Type<'a>>),
    Ref(&'a str),
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[cfg_attr(feature = "to_json", serde(rename_all = "lowercase"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item<'a> {
    #[cfg_attr(feature = "to_json", serde(serialize_with = "ser::serialize_enum"))]
    Enum(Vec<Variant<'a>>),
    Properties(Vec<Param<'a>>),
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
    pub description: Description<'a>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub experimental: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub deprecated: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub optional: bool,
    #[cfg_attr(feature = "to_json", serde(flatten))]
    pub ty: Type<'a>,
    pub name: &'a str,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
    pub description: Description<'a>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub experimental: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub deprecated: bool,
    pub name: &'a str,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "to_json", serde(serialize_with = "ser::serialize_redirect"))]
    pub redirect: Option<Redirect<'a>>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub parameters: Vec<Param<'a>>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub returns: Vec<Param<'a>>,
}

#[cfg_attr(feature = "to_json", derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event<'a> {
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(
        feature = "to_json",
        serde(serialize_with = "ser::serialize_description")
    )]
    pub description: Description<'a>,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub experimental: bool,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "ser::is_false"))]
    pub deprecated: bool,
    pub name: &'a str,
    #[cfg_attr(feature = "to_json", serde(skip_serializing_if = "Vec::is_empty"))]
    pub parameters: Vec<Param<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirect<'a> {
    pub description: Description<'a>,
    pub to: &'a str,
}
