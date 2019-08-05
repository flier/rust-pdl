use std::ops::RangeFrom;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take_until, take_while},
    character::complete::char,
    combinator::{map, map_res, opt, recognize, verify},
    error::ParseError,
    multi::{many0, many1},
    sequence::{pair, preceded, tuple},
    AsChar, Compare, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Slice,
};

use crate::*;

/// Parse a `Protocol` from a string of PDL format.
pub fn parse(input: &str) -> IResult<&str, Protocol> {
    protocol(input)
}

fn protocol(input: &str) -> IResult<&str, Protocol> {
    map(
        tuple((
            description,
            empty_lines,
            version,
            many1(preceded(empty_lines, domain)),
        )),
        |(description, _, version, domains)| Protocol {
            description,
            version,
            domains,
        },
    )(input)
}

fn indent(input: &str) -> IResult<&str, &str> {
    recognize(many1(is_a(" \t")))(input)
}

fn empty_lines(input: &str) -> IResult<&str, &str> {
    recognize(many0(eol))(input)
}

fn eol(input: &str) -> IResult<&str, char> {
    char('\n')(input)
}

fn description(input: &str) -> IResult<&str, Description> {
    map(many0(comment), Description)(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    map(
        tuple((
            opt(indent),
            char('#'),
            map(take_until("\n"), str::trim),
            eol,
        )),
        |(_, _, s, _eol)| s,
    )(input)
}

fn version(input: &str) -> IResult<&str, Version> {
    map(
        tuple((
            tuple((tag("version"), eol)),
            tuple((
                indent,
                tag("major"),
                char(' '),
                map_res(take_while(|c: char| c.is_ascii_digit()), FromStr::from_str),
                eol,
            )),
            tuple((
                indent,
                tag("minor"),
                char(' '),
                map_res(take_while(|c: char| c.is_ascii_digit()), FromStr::from_str),
                eol,
            )),
        )),
        |((_version, _), (_, _major, _, major, _), (_, _minor, _, minor, _))| Version {
            major,
            minor,
        },
    )(input)
}

fn domain(input: &str) -> IResult<&str, Domain> {
    enum Item<'a> {
        TypeDef(TypeDef<'a>),
        Command(Command<'a>),
        Event(Event<'a>),
    }

    map(
        tuple((
            description,
            tuple((
                optional("experimental"),
                optional("deprecated"),
                tag("domain"),
                char(' '),
                take_until("\n"),
                eol,
            )),
            many0(depends_on),
            many0(preceded(
                empty_lines,
                alt((
                    map(type_def, Item::TypeDef),
                    map(command, Item::Command),
                    map(event, Item::Event),
                )),
            )),
        )),
        |(description, (experimental, deprecated, _domain, _, name, _eol), dependencies, items)| {
            let (types, commands, events) = items.into_iter().fold(
                (vec![], vec![], vec![]),
                |(mut types, mut commands, mut events), item| {
                    match item {
                        Item::TypeDef(ty) => types.push(ty),
                        Item::Command(cmd) => commands.push(cmd),
                        Item::Event(evt) => events.push(evt),
                    }

                    (types, commands, events)
                },
            );

            Domain {
                description,
                experimental,
                deprecated,
                name,
                dependencies,
                types,
                commands,
                events,
            }
        },
    )(input)
}

fn depends_on(input: &str) -> IResult<&str, &str> {
    map(
        tuple((
            indent,
            tag("depends on"),
            char(' '),
            take_while(|c: char| !c.is_whitespace()),
            eol,
        )),
        |(_, _depends_on, _, name, _eol)| name,
    )(input)
}

fn type_def(input: &str) -> IResult<&str, TypeDef> {
    map(
        tuple((
            description,
            tuple((
                indent,
                optional("experimental"),
                optional("deprecated"),
                tag("type"),
                char(' '),
                take_while(|c: char| !c.is_whitespace()),
                char(' '),
                tag("extends"),
                char(' '),
                ty,
                eol,
            )),
            opt(item),
        )),
        |(
            description,
            (_, experimental, deprecated, _type, _, id, _, _extends, _, extends, _),
            item,
        )| {
            let ty = TypeDef {
                description,
                experimental,
                deprecated,
                id,
                extends,
                item,
            };

            trace!("{:?}", ty);

            ty
        },
    )(input)
}

fn ty(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            optional("array of"),
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(is_array, ty)| Type::new(ty, is_array),
    )(input)
}

impl Type<'_> {
    fn new(ty: &str, is_array: bool) -> Type {
        if is_array {
            Type::ArrayOf(Box::new(Type::new(ty, false)))
        } else {
            match ty {
                "enum" => Type::Enum(vec![]),
                "integer" => Type::Integer,
                "number" => Type::Number,
                "boolean" => Type::Boolean,
                "string" => Type::String,
                "object" => Type::Object,
                "any" => Type::Any,
                _ => Type::Ref(ty),
            }
        }
    }
}

fn item(input: &str) -> IResult<&str, Item> {
    alt((
        map(
            preceded(tuple((indent, tag("enum"), eol)), many1(variant)),
            Item::Enum,
        ),
        map(
            preceded(tuple((indent, tag("properties"), eol)), many1(param)),
            Item::Properties,
        ),
    ))(input)
}

fn variant(input: &str) -> IResult<&str, Variant> {
    map(
        tuple((
            description,
            tuple((
                indent,
                verify(take_while(|c: char| !c.is_whitespace()), |s: &str| {
                    !s.is_empty()
                }),
                eol,
            )),
        )),
        |(description, (_, name, _))| {
            let variant = Variant { description, name };

            trace!("{:?}", variant);

            variant
        },
    )(input)
}

fn param(input: &str) -> IResult<&str, Param> {
    let (input, mut param) = map(
        tuple((
            description,
            tuple((
                indent,
                optional("experimental"),
                optional("deprecated"),
                optional("optional"),
                ty,
                char(' '),
                verify(take_while(|c: char| !c.is_whitespace()), |s: &str| {
                    !s.is_empty()
                }),
                eol,
            )),
        )),
        |(description, (_, experimental, deprecated, optional, ty, _, name, _))| {
            let param = Param {
                experimental,
                deprecated,
                optional,
                ty,
                description,
                name,
            };

            trace!("{:?}", param);

            param
        },
    )(input)?;

    if let Type::Enum(ref mut variants) = param.ty {
        let (input, mut vars) = many1(variant)(input)?;

        trace!("{:?}", vars);

        variants.append(&mut vars);

        Ok((input, param))
    } else {
        Ok((input, param))
    }
}

fn command(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            description,
            tuple((
                indent,
                optional("experimental"),
                optional("deprecated"),
                tag("command"),
                char(' '),
                take_until("\n"),
                eol,
            )),
            opt(redirect),
            opt(preceded(
                tuple((indent, tag("parameters"), eol)),
                many1(param),
            )),
            empty_lines,
            opt(preceded(tuple((indent, tag("returns"), eol)), many1(param))),
        )),
        |(
            description,
            (_, experimental, deprecated, _, _, name, _),
            redirect,
            parameters,
            _,
            returns,
        )| {
            let command = Command {
                description,
                experimental,
                deprecated,
                name,
                redirect,
                parameters: parameters.unwrap_or_default(),
                returns: returns.unwrap_or_default(),
            };

            trace!("{:?}", command);

            command
        },
    )(input)
}

fn event(input: &str) -> IResult<&str, Event> {
    map(
        tuple((
            description,
            tuple((
                indent,
                optional("experimental"),
                optional("deprecated"),
                tag("event"),
                char(' '),
                take_until("\n"),
                eol,
            )),
            opt(preceded(
                tuple((indent, tag("parameters"), eol)),
                many1(param),
            )),
        )),
        |(description, (_, experimental, deprecated, _, _, name, _), parameters)| {
            let event = Event {
                description,
                experimental,
                deprecated,
                name,
                parameters: parameters.unwrap_or_default(),
            };

            trace!("{:?}", event);

            event
        },
    )(input)
}

fn redirect(input: &str) -> IResult<&str, Redirect> {
    map(
        tuple((
            description,
            tuple((
                indent,
                tag("redirect"),
                char(' '),
                take_while(|c: char| !c.is_whitespace()),
                eol,
            )),
        )),
        |(description, (_, _redirect, _, to, _))| {
            let redirect = Redirect { description, to };

            trace!("{:?}", redirect);

            redirect
        },
    )(input)
}

fn optional<T, I, E>(name: T) -> impl Fn(I) -> IResult<I, bool, E>
where
    T: InputLength + InputIter + Clone,
    I: Slice<RangeFrom<usize>> + InputTake + InputTakeAtPosition + InputIter + Compare<T> + Clone,
    <I as InputIter>::Item: AsChar,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    E: ParseError<I>,
{
    map(opt(pair(tag(name), char(' '))), |v| v.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_protocol() {
        assert_eq!(
            protocol(
                r#"# Copyright 2017 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

version
  major 1
  minor 3

experimental domain Accessibility
  depends on DOM

  # Unique accessibility node identifier.
  type AXNodeId extends string

  # Enum of possible property types.
  type AXValueType extends string
    enum
      boolean
      tristate
      booleanOrUndefined

  # A single source for a computed AX property.
  type AXValueSource extends object
    properties
      # What type of source this is.
      AXValueSourceType type
      # The value of this property source.
      optional AXValue value
      # The name of the relevant attribute, if any.
      optional string attribute
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
                    ]
                    .into(),
                    version: Version { major: 1, minor: 3 },
                    domains: vec![Domain {
                        description: Default::default(),
                        experimental: true,
                        deprecated: false,
                        name: "Accessibility",
                        dependencies: vec!["DOM"],
                        types: vec![
                            TypeDef {
                                description: "Unique accessibility node identifier.".into(),
                                experimental: false,
                                deprecated: false,
                                id: "AXNodeId",
                                extends: Type::String,
                                item: None,
                            },
                            TypeDef {
                                description: "Enum of possible property types.".into(),
                                experimental: false,
                                deprecated: false,
                                id: "AXValueType",
                                extends: Type::String,
                                item: Some(Item::Enum(vec![
                                    Variant {
                                        description: Default::default(),
                                        name: "boolean"
                                    },
                                    Variant {
                                        description: Default::default(),
                                        name: "tristate"
                                    },
                                    Variant {
                                        description: Default::default(),
                                        name: "booleanOrUndefined"
                                    }
                                ]))
                            },
                            TypeDef {
                                description: "A single source for a computed AX property.".into(),
                                experimental: false,
                                deprecated: false,
                                id: "AXValueSource",
                                extends: Type::Object,
                                item: Some(Item::Properties(vec![
                                    Param {
                                        description: "What type of source this is.".into(),
                                        experimental: false,
                                        deprecated: false,
                                        optional: false,
                                        ty: Type::Ref("AXValueSourceType"),
                                        name: "type"
                                    },
                                    Param {
                                        description: "The value of this property source.".into(),
                                        experimental: false,
                                        deprecated: false,
                                        optional: true,
                                        ty: Type::Ref("AXValue"),
                                        name: "value"
                                    },
                                    Param {
                                        description: "The name of the relevant attribute, if any."
                                            .into(),
                                        experimental: false,
                                        deprecated: false,
                                        optional: true,
                                        ty: Type::String,
                                        name: "attribute"
                                    }
                                ]))
                            }
                        ],
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
            ("", Version { major: 1, minor: 3 })
        )
    }

    #[test]
    fn parse_domain() {
        assert_eq!(
            domain("experimental domain Accessibility\n").unwrap(),
            (
                "",
                Domain {
                    description: Default::default(),
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
        assert_eq!(depends_on("  depends on DOM\n").unwrap(), ("", "DOM"));
    }

    #[test]
    fn parse_type_def() {
        assert_eq!(
            type_def(
                r#"  type AXProperty extends object
    properties
      # The name of this property.
      AXPropertyName name
      # The value of this property.
      AXValue value
"#
            )
            .unwrap(),
            (
                "",
                TypeDef {
                    description: Default::default(),
                    experimental: false,
                    deprecated: false,
                    id: "AXProperty",
                    extends: Type::Object,
                    item: Some(Item::Properties(vec![
                        Param {
                            description: "The name of this property.".into(),
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Type::Ref("AXPropertyName"),
                            name: "name"
                        },
                        Param {
                            description: "The value of this property.".into(),
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Type::Ref("AXValue"),
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
            type_def(
                r#"  # Enum of possible property sources.
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
                "",
                TypeDef {
                    description: "Enum of possible property sources.".into(),
                    experimental: false,
                    deprecated: false,
                    id: "AXValueSourceType",
                    extends: Type::String,
                    item: Some(Item::Enum(vec![
                        Variant {
                            description: Default::default(),
                            name: "attribute"
                        },
                        Variant {
                            description: Default::default(),
                            name: "implicit"
                        },
                        Variant {
                            description: Default::default(),
                            name: "style"
                        },
                        Variant {
                            description: Default::default(),
                            name: "contents"
                        },
                        Variant {
                            description: Default::default(),
                            name: "placeholder"
                        },
                        Variant {
                            description: Default::default(),
                            name: "relatedElement"
                        }
                    ]))
                }
            )
        );

        assert_eq!(
            type_def(
                r#"  # Pseudo element type.
  type PseudoType extends string
    enum
      first-line
      first-letter
      before
"#
            )
            .unwrap(),
            (
                "",
                TypeDef {
                    description: "Pseudo element type.".into(),
                    experimental: false,
                    deprecated: false,
                    id: "PseudoType",
                    extends: Type::String,
                    item: Some(Item::Enum(vec![
                        Variant {
                            description: Default::default(),
                            name: "first-line"
                        },
                        Variant {
                            description: Default::default(),
                            name: "first-letter"
                        },
                        Variant {
                            description: Default::default(),
                            name: "before"
                        }
                    ]))
                }
            )
        );
    }

    #[test]
    fn parse_params() {
        assert_eq!(
            item(
                r#"    properties
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
                "",
                Item::Properties(vec![
                    Param {
                        description: "The type of this value.".into(),
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Type::Ref("AXValueType"),
                        name: "type"
                    },
                    Param {
                        description: "The computed value of this property.".into(),
                        experimental: false,
                        deprecated: false,
                        optional: true,
                        ty: Type::Any,
                        name: "value"
                    },
                    Param {
                        description: "One or more related nodes, if applicable.".into(),
                        experimental: false,
                        deprecated: false,
                        optional: true,
                        ty: Type::ArrayOf(Box::new(Type::Ref("AXRelatedNode"))),
                        name: "relatedNodes"
                    },
                    Param {
                        description: "Animation type of `Animation`.".into(),
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Type::Enum(vec![
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
                r#"  # Returns the DER-encoded certificate.
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
                "",
                Command {
                    description: "Returns the DER-encoded certificate.".into(),
                    experimental: true,
                    deprecated: false,
                    name: "getCertificate",
                    redirect: None,
                    parameters: vec![Param {
                        description: "Origin to get certificate for.".into(),
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Type::String,
                        name: "origin"
                    }],
                    returns: vec![Param {
                        description: Default::default(),
                        experimental: false,
                        deprecated: false,
                        optional: false,
                        ty: Type::ArrayOf(Box::new(Type::String)),
                        name: "tableNames"
                    }],
                }
            )
        );

        assert_eq!(
            command(
                r#"  # Hides any highlight.
  command hideHighlight
    # Use 'Overlay.hideHighlight' instead
    redirect Overlay
"#
            )
            .unwrap(),
            (
                "",
                Command {
                    description: "Hides any highlight.".into(),
                    experimental: false,
                    deprecated: false,
                    name: "hideHighlight",
                    redirect: Some(Redirect {
                        description: "Use 'Overlay.hideHighlight' instead".into(),
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
            event(r#"  # Notification sent after the virtual time has advanced.
  experimental event virtualTimeAdvanced
    parameters
      # The amount of virtual time that has elapsed in milliseconds since virtual time was first
      # enabled.
      number virtualTimeElapsed
"#).unwrap(),
            (
                "",
                Event {
                    description: "Notification sent after the virtual time has advanced.".into(),
                    experimental: true,
                    deprecated: false,
                    name: "virtualTimeAdvanced",
                    parameters: vec![
                        Param {
                            description: vec![
                                "The amount of virtual time that has elapsed in milliseconds since virtual time was first",
                                "enabled."
                            ].into(),
                            experimental: false,
                            deprecated: false,
                            optional: false,
                            ty: Type::Number,
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
                r#"    # Use 'Emulation.clearGeolocationOverride' instead
    redirect Emulation
"#
            )
            .unwrap(),
            (
                "",
                Redirect {
                    description: "Use 'Emulation.clearGeolocationOverride' instead".into(),
                    to: "Emulation"
                }
            )
        )
    }
}
