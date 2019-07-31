use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{char, multispace0, multispace1, space1},
    combinator::{map, map_res, opt},
    error::ParseError,
    multi::{many0, many1},
    sequence::{pair, preceded, tuple},
    AsChar, Compare, IResult, InputLength, InputTake, InputTakeAtPosition,
};
use std::str::FromStr;

pub type Description<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub struct Protocol<'a> {
    description: Description<'a>,
    version: (usize, usize),
}

fn protocol(input: &str) -> IResult<&str, Protocol> {
    map(
        tuple((description, version, multispace1)),
        |(description, version, _)| Protocol {
            description,
            version,
        },
    )(input)
}

fn description(input: &str) -> IResult<&str, Description> {
    map(
        many0(alt((map(comment, Some), map(multispace1, |_| None)))),
        |lines| lines.into_iter().flatten().collect::<Vec<_>>(),
    )(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    map(
        preceded(pair(multispace0, char('#')), take_until("\n")),
        str::trim,
    )(input)
}

fn version(input: &str) -> IResult<&str, (usize, usize)> {
    map(
        tuple((
            tag("version"),
            multispace1,
            tag("major"),
            space1,
            map_res(take_while(|c: char| c.is_ascii_digit()), FromStr::from_str),
            multispace1,
            tag("minor"),
            space1,
            map_res(take_while(|c: char| c.is_ascii_digit()), FromStr::from_str),
        )),
        |(_version, _, _major, _, major, _, _minor, _, minor)| (major, minor),
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Domain<'a> {
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub dependencies: Vec<&'a str>,
    pub types: Vec<Type<'a>>,
    pub commands: Vec<Command<'a>>,
    pub events: Vec<Event<'a>>,
}

fn domain(input: &str) -> IResult<&str, Domain> {
    map(
        tuple((
            optional("experimental"),
            optional("deprecated"),
            tag("domain"),
            space1,
            take_until("\n"),
            many0(depends_on),
            many0(type_),
            many0(command),
            many0(event),
        )),
        |(experimental, deprecated, _, _, name, dependencies, types, commands, events)| Domain {
            experimental,
            deprecated,
            name,
            dependencies,
            types,
            commands,
            events,
        },
    )(input)
}

fn depends_on(input: &str) -> IResult<&str, &str> {
    map(
        tuple((
            multispace1,
            tag("depends on"),
            space1,
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(_, _, _, name)| name,
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Type<'a> {
    pub experimental: bool,
    pub deprecated: bool,
    pub optional: bool,
    pub id: &'a str,
    pub extends: Ty<'a>,
    pub items: Vec<Item<'a>>,
}

fn type_(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            optional("optional"),
            tag("type"),
            space1,
            take_while(|c: char| !c.is_whitespace()),
            space1,
            tag("extends"),
            space1,
            ty,
            multispace1,
            many0(item),
        )),
        |(
            _,
            experimental,
            deprecated,
            optional,
            _type,
            _,
            id,
            _,
            _extends,
            _,
            extends,
            _,
            items,
        )| Type {
            experimental,
            deprecated,
            optional,
            id,
            extends,
            items,
        },
    )(input)
}

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

fn ty(input: &str) -> IResult<&str, Ty> {
    map(
        tuple((
            optional("array of"),
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(is_array, ty)| Ty::new(ty, is_array),
    )(input)
}

#[derive(Debug, PartialEq)]
pub enum Item<'a> {
    Enum(Vec<Variant<'a>>),
    Parameters(Vec<Param<'a>>),
    Returns(Vec<Param<'a>>),
    Properties(Vec<Param<'a>>),
}

fn item(input: &str) -> IResult<&str, Item> {
    preceded(
        multispace0,
        alt((
            map(preceded(tag("enum"), many1(variant)), Item::Enum),
            map(preceded(tag("parameters"), many1(param)), Item::Parameters),
            map(preceded(tag("returns"), many1(param)), Item::Returns),
            map(preceded(tag("properties"), many1(param)), Item::Properties),
        )),
    )(input)
}

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

fn variant(input: &str) -> IResult<&str, Variant> {
    map(
        tuple((description, take_until("\n"))),
        |(description, name)| Variant { description, name },
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Param<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub optional: bool,
    pub ty: Ty<'a>,
    pub name: &'a str,
}

fn param(input: &str) -> IResult<&str, Param> {
    let (input, mut param) = map(
        tuple((
            description,
            optional("experimental"),
            optional("deprecated"),
            optional("optional"),
            ty,
            space1,
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(description, experimental, optional, deprecated, ty, _, name)| Param {
            experimental,
            optional,
            deprecated,
            ty,
            description,
            name,
        },
    )(input)?;

    if let Ty::Enum(ref mut variants) = param.ty {
        let (input, mut vars) = many1(variant)(input)?;

        variants.append(&mut vars);

        Ok((input, param))
    } else {
        Ok((input, param))
    }
}

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
}

fn command(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            tag("command"),
            space1,
            take_until("\n"),
        )),
        |(_, experimental, deprecated, _, _, name)| Command {
            experimental,
            deprecated,
            name,
        },
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Event<'a> {
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
}

fn event(input: &str) -> IResult<&str, Event> {
    map(
        tuple((
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            tag("event"),
            space1,
            take_until("\n"),
        )),
        |(_, experimental, deprecated, _, _, name)| Event {
            experimental,
            deprecated,
            name,
        },
    )(input)
}

fn optional<T, I, E>(name: T) -> impl Fn(I) -> IResult<I, bool, E>
where
    T: InputLength + Clone,
    I: InputTake + InputTakeAtPosition + Compare<T> + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I>,
{
    map(opt(pair(tag(name), space1)), |v| v.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_protocol() {
        assert_eq!(
            protocol(
                r#"
# Copyright 2017 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

version
  major 1
  minor 3
"#
            )
            .unwrap(),
            (
                "",
                Protocol {
                    description: vec![
                        "Copyright 2017 The Chromium Authors. All rights reserved.",
                        "Use of this source code is governed by a BSD-style license that can be",
                        "found in the LICENSE file."
                    ],
                    version: (1, 3),
                }
            )
        )
    }

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment("# Copyright 2017 The Chromium Authors. All rights reserved.\n").unwrap(),
            (
                "\n",
                "Copyright 2017 The Chromium Authors. All rights reserved."
            )
        )
    }

    #[test]
    fn parse_version() {
        assert_eq!(
            version(
                r#"version
  major 1
  minor 3
"#
            )
            .unwrap(),
            ("\n", (1, 3))
        )
    }

    #[test]
    fn parse_domain() {
        assert_eq!(
            domain("experimental domain Accessibility\n").unwrap(),
            (
                "\n",
                Domain {
                    experimental: true,
                    deprecated: false,
                    name: "Accessibility",
                    dependencies: vec![],
                    types: vec![],
                    commands: vec![],
                    events: vec![],
                }
            )
        );
    }

    #[test]
    fn parse_depends_on() {
        assert_eq!(depends_on("  depends on DOM\n").unwrap(), ("\n", "DOM"));
    }

    #[test]
    fn parse_type() {
        assert_eq!(
            type_("  type AXNodeId extends string\n").unwrap(),
            (
                "",
                Type {
                    experimental: false,
                    deprecated: false,
                    optional: false,
                    id: "AXNodeId",
                    extends: Ty::String,
                    items: vec![]
                }
            )
        )
    }

    #[test]
    fn parse_enum() {
        assert_eq!(
            item(
                r#"
    enum
      attribute
      implicit
      style
"#
            )
            .unwrap(),
            (
                "\n",
                Item::Enum(vec![
                    Variant::new("attribute"),
                    Variant::new("implicit"),
                    Variant::new("style")
                ])
            )
        )
    }

    #[test]
    fn parse_params() {
        assert_eq!(
            item(
                r#"
    properties
      # The type of this value.
      AXValueType type
      # The computed value of this property.
      optional any value
      # One or more related nodes, if applicable.
      optional array of AXRelatedNode relatedNodes
      # Animation type of `Animation`.
      enum type
        CSSTransition
        CSSAnimation
        WebAnimation
"#
            )
            .unwrap(),
            (
                "\n",
                Item::Properties(vec![
                    Param {
                        description: vec!["The type of this value."],
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Ty::Ref("AXValueType"),
                        name: "type"
                    },
                    Param {
                        description: vec!["The computed value of this property."],
                        experimental: false,
                        deprecated: true,
                        optional: false,
                        ty: Ty::Any,
                        name: "value"
                    },
                    Param {
                        description: vec!["One or more related nodes, if applicable."],
                        experimental: false,
                        deprecated: true,
                        optional: false,
                        ty: Ty::ArrayOf(Box::new(Ty::Ref("AXRelatedNode"))),
                        name: "relatedNodes"
                    },
                    Param {
                        description: vec!["Animation type of `Animation`."],
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Ty::Enum(vec![
                            Variant::new("CSSTransition"),
                            Variant::new("CSSAnimation"),
                            Variant::new("WebAnimation"),
                        ]),
                        name: "type"
                    }
                ])
            )
        )
    }

    #[test]
    fn parse_command() {
        assert_eq!(
            command("  command disable\n").unwrap(),
            (
                "\n",
                Command {
                    experimental: false,
                    deprecated: false,
                    name: "disable"
                }
            )
        );
    }

    #[test]
    fn parse_event() {
        assert_eq!(
            event("  event animationCanceled\n").unwrap(),
            (
                "\n",
                Event {
                    experimental: false,
                    deprecated: false,
                    name: "animationCanceled"
                }
            )
        );
    }
}
