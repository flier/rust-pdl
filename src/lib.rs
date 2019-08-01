use std::str::FromStr;

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

pub fn parse(input: &str) -> IResult<&str, Protocol> {
    protocol(input)
}

pub type Description<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub struct Protocol<'a> {
    pub description: Description<'a>,
    pub version: (usize, usize),
    pub domains: Vec<Domain<'a>>,
}

fn protocol(input: &str) -> IResult<&str, Protocol> {
    map(
        tuple((description, multispace1, version, many1(domain))),
        |(description, _, version, domains)| Protocol {
            description,
            version,
            domains,
        },
    )(input)
}

fn description(input: &str) -> IResult<&str, Description> {
    many0(comment)(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    map(
        tuple((
            multispace0,
            char('#'),
            map(take_until("\n"), str::trim),
            char('\n'),
        )),
        |(_, _, s, _)| s,
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
    pub description: Description<'a>,
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
            description,
            multispace0,
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
        |(
            description,
            _,
            experimental,
            deprecated,
            _,
            _,
            name,
            dependencies,
            types,
            commands,
            events,
        )| Domain {
            description,
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
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub optional: bool,
    pub id: &'a str,
    pub extends: Ty<'a>,
    pub item: Option<Item<'a>>,
}

fn type_(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            description,
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
            opt(item),
        )),
        |(
            description,
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
            item,
        )| Type {
            description,
            experimental,
            deprecated,
            optional,
            id,
            extends,
            item,
        },
    )(input)
}

#[derive(Debug, PartialEq)]
pub enum Item<'a> {
    Enum(Vec<Variant<'a>>),
    Properties(Vec<Param<'a>>),
}

fn item(input: &str) -> IResult<&str, Item> {
    preceded(
        multispace0,
        alt((
            map(preceded(tag("enum"), many1(variant)), Item::Enum),
            map(preceded(tag("properties"), many1(param)), Item::Properties),
        )),
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
        tuple((description, multispace1, take_until("\n"))),
        |(description, _, name)| Variant { description, name },
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
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            optional("optional"),
            ty,
            space1,
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(description, _, experimental, optional, deprecated, ty, _, name)| Param {
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
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub redirect: Option<Redirect<'a>>,
    pub parameters: Vec<Param<'a>>,
    pub returns: Vec<Param<'a>>,
}

fn command(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            description,
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            tag("command"),
            space1,
            take_until("\n"),
            opt(redirect),
            opt(preceded(pair(multispace1, tag("parameters")), many1(param))),
            opt(preceded(pair(multispace1, tag("returns")), many1(param))),
        )),
        |(description, _, experimental, deprecated, _, _, name, redirect, parameters, returns)| {
            Command {
                description,
                experimental,
                deprecated,
                name,
                redirect,
                parameters: parameters.unwrap_or_default(),
                returns: returns.unwrap_or_default(),
            }
        },
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Event<'a> {
    pub description: Description<'a>,
    pub experimental: bool,
    pub deprecated: bool,
    pub name: &'a str,
    pub parameters: Vec<Param<'a>>,
}

fn event(input: &str) -> IResult<&str, Event> {
    map(
        tuple((
            description,
            multispace1,
            optional("experimental"),
            optional("deprecated"),
            tag("event"),
            space1,
            take_until("\n"),
            opt(preceded(pair(multispace1, tag("parameters")), many1(param))),
        )),
        |(description, _, experimental, deprecated, _, _, name, parameters)| Event {
            description,
            experimental,
            deprecated,
            name,
            parameters: parameters.unwrap_or_default(),
        },
    )(input)
}

#[derive(Debug, PartialEq)]
pub struct Redirect<'a> {
    pub description: Description<'a>,
    pub to: &'a str,
}

fn redirect(input: &str) -> IResult<&str, Redirect> {
    map(
        tuple((
            description,
            multispace1,
            tag("redirect"),
            space1,
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(description, _, _redirect, _, to)| Redirect { description, to },
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

experimental domain Accessibility
  depends on DOM

  # Unique accessibility node identifier.
  type AXNodeId extends string
"#
            )
            .unwrap(),
            (
                "\n",
                Protocol {
                    description: vec![
                        "Copyright 2017 The Chromium Authors. All rights reserved.",
                        "Use of this source code is governed by a BSD-style license that can be",
                        "found in the LICENSE file."
                    ],
                    version: (1, 3),
                    domains: vec![Domain {
                        description: vec![],
                        experimental: true,
                        deprecated: false,
                        name: "Accessibility",
                        dependencies: vec!["DOM"],
                        types: vec![Type {
                            description: vec!["Unique accessibility node identifier."],
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            id: "AXNodeId",
                            extends: Ty::String,
                            item: None
                        }],
                        commands: vec![],
                        events: vec![]
                    }],
                }
            )
        )
    }

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment("# Copyright 2017 The Chromium Authors. All rights reserved.\n").unwrap(),
            (
                "",
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
                    description: vec![],
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
            type_(
                r#"
  type AXProperty extends object
    properties
      # The name of this property.
      AXPropertyName name
      # The value of this property.
      AXValue value
"#
            )
            .unwrap(),
            (
                "\n",
                Type {
                    description: vec![],
                    experimental: false,
                    deprecated: false,
                    optional: false,
                    id: "AXProperty",
                    extends: Ty::Object,
                    item: Some(Item::Properties(vec![
                        Param {
                            description: vec!["The name of this property."],
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Ty::Ref("AXPropertyName"),
                            name: "name"
                        },
                        Param {
                            description: vec!["The value of this property."],
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Ty::Ref("AXValue"),
                            name: "value"
                        }
                    ]))
                }
            )
        )
    }

    #[test]
    fn parse_enum() {
        assert_eq!(
            type_(
                r#"
  # Enum of possible property sources.
  type AXValueSourceType extends string
    enum
      attribute
      implicit
      style
      contents
      placeholder
      relatedElement
"#
            )
            .unwrap(),
            (
                "\n",
                Type {
                    description: vec!["Enum of possible property sources."],
                    experimental: false,
                    deprecated: false,
                    optional: false,
                    id: "AXValueSourceType",
                    extends: Ty::String,
                    item: Some(Item::Enum(vec![
                        Variant {
                            description: vec![],
                            name: "attribute"
                        },
                        Variant {
                            description: vec![],
                            name: "implicit"
                        },
                        Variant {
                            description: vec![],
                            name: "style"
                        },
                        Variant {
                            description: vec![],
                            name: "contents"
                        },
                        Variant {
                            description: vec![],
                            name: "placeholder"
                        },
                        Variant {
                            description: vec![],
                            name: "relatedElement"
                        }
                    ]))
                }
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
            command(
                r#"
  # Returns the DER-encoded certificate.
  experimental command getCertificate
    parameters
      # Origin to get certificate for.
      string origin
    returns
      array of string tableNames
"#
            )
            .unwrap(),
            (
                "\n",
                Command {
                    description: vec!["Returns the DER-encoded certificate."],
                    experimental: true,
                    deprecated: false,
                    name: "getCertificate",
                    redirect: None,
                    parameters: vec![Param {
                        description: vec!["Origin to get certificate for."],
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Ty::String,
                        name: "origin"
                    }],
                    returns: vec![Param {
                        description: vec![],
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Ty::ArrayOf(Box::new(Ty::String)),
                        name: "tableNames"
                    }],
                }
            )
        );

        assert_eq!(
            command(
                r#"
  # Hides any highlight.
  command hideHighlight
    # Use 'Overlay.hideHighlight' instead
    redirect Overlay
"#
            )
            .unwrap(),
            (
                "\n",
                Command {
                    description: vec!["Hides any highlight."],
                    experimental: false,
                    deprecated: false,
                    name: "hideHighlight",
                    redirect: Some(Redirect {
                        description: vec!["Use 'Overlay.hideHighlight' instead"],
                        to: "Overlay"
                    }),
                    parameters: vec![],
                    returns: vec![],
                }
            )
        );
    }

    #[test]
    fn parse_event() {
        assert_eq!(
            event(r#"
  # Notification sent after the virtual time has advanced.
  experimental event virtualTimeAdvanced
    parameters
      # The amount of virtual time that has elapsed in milliseconds since virtual time was first
      # enabled.
      number virtualTimeElapsed
"#).unwrap(),
            (
                "\n",
                Event {
                    description: vec!["Notification sent after the virtual time has advanced."],
                    experimental: true,
                    deprecated: false,
                    name: "virtualTimeAdvanced",
                    parameters: vec![
                        Param {
                            description: vec![
                                "The amount of virtual time that has elapsed in milliseconds since virtual time was first",
                                "enabled."
                            ],
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Ty::Number,
                            name: "virtualTimeElapsed"
                        },
                    ],
                }
            )
        );
    }

    #[test]
    fn parse_redirect() {
        assert_eq!(
            redirect(
                r#"
    # Use 'Emulation.clearGeolocationOverride' instead
    redirect Emulation
"#
            )
            .unwrap(),
            (
                "\n",
                Redirect {
                    description: vec!["Use 'Emulation.clearGeolocationOverride' instead"],
                    to: "Emulation"
                }
            )
        )
    }
}
