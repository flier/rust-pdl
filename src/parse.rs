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
    many0(comment)(input)
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
            many0(preceded(empty_lines, type_)),
            many0(preceded(empty_lines, command)),
            many0(preceded(empty_lines, event)),
        )),
        |(
            description,
            (experimental, deprecated, _domain, _, domain, _eol),
            dependencies,
            types,
            commands,
            events,
        )| Domain {
            description,
            experimental,
            deprecated,
            domain,
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
            indent,
            tag("depends on"),
            char(' '),
            take_while(|c: char| !c.is_whitespace()),
            eol,
        )),
        |(_, _depends_on, _, name, _eol)| name,
    )(input)
}

fn type_(input: &str) -> IResult<&str, Type> {
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
            let ty = Type {
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

fn ty(input: &str) -> IResult<&str, Ty> {
    map(
        tuple((
            optional("array of"),
            take_while(|c: char| !c.is_whitespace()),
        )),
        |(is_array, ty)| Ty::new(ty, is_array),
    )(input)
}

impl Ty<'_> {
    fn new(ty: &str, is_array: bool) -> Ty {
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

    if let Ty::Enum(ref mut variants) = param.ty {
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
            opt(preceded(tuple((indent, tag("returns"), eol)), many1(param))),
        )),
        |(
            description,
            (_, experimental, deprecated, _, _, name, _),
            redirect,
            parameters,
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
                    ],
                    version: Version { major: 1, minor: 3 },
                    domains: vec![Domain {
                        description: vec![],
                        experimental: true,
                        deprecated: false,
                        domain: "Accessibility",
                        dependencies: vec!["DOM"],
                        types: vec![
                            Type {
                                description: vec!["Unique accessibility node identifier."],
                                experimental: false,
                                deprecated: false,
                                id: "AXNodeId",
                                extends: Ty::String,
                                item: None,
                            },
                            Type {
                                description: vec!["Enum of possible property types."],
                                experimental: false,
                                deprecated: false,
                                id: "AXValueType",
                                extends: Ty::String,
                                item: Some(Item::Enum(vec![
                                    Variant {
                                        description: vec![],
                                        name: "boolean"
                                    },
                                    Variant {
                                        description: vec![],
                                        name: "tristate"
                                    },
                                    Variant {
                                        description: vec![],
                                        name: "booleanOrUndefined"
                                    }
                                ]))
                            },
                            Type {
                                description: vec!["A single source for a computed AX property."],
                                experimental: false,
                                deprecated: false,
                                id: "AXValueSource",
                                extends: Ty::Object,
                                item: Some(Item::Properties(vec![
                                    Param {
                                        description: vec!["What type of source this is."],
                                        experimental: false,
                                        deprecated: false,
                                        optional: false,
                                        ty: Ty::Ref("AXValueSourceType"),
                                        name: "type"
                                    },
                                    Param {
                                        description: vec!["The value of this property source."],
                                        experimental: false,
                                        deprecated: false,
                                        optional: true,
                                        ty: Ty::Ref("AXValue"),
                                        name: "value"
                                    },
                                    Param {
                                        description: vec![
                                            "The name of the relevant attribute, if any."
                                        ],
                                        experimental: false,
                                        deprecated: false,
                                        optional: true,
                                        ty: Ty::String,
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
                    description: vec![],
                    experimental: true,
                    deprecated: false,
                    domain: "Accessibility",
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
    fn parse_type() {
        assert_eq!(
            type_(
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
                Type {
                    description: vec![],
                    experimental: false,
                    deprecated: false,
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
                Type {
                    description: vec!["Enum of possible property sources."],
                    experimental: false,
                    deprecated: false,
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
        );

        assert_eq!(
            type_(
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
                Type {
                    description: vec!["Pseudo element type."],
                    experimental: false,
                    deprecated: false,
                    id: "PseudoType",
                    extends: Ty::String,
                    item: Some(Item::Enum(vec![
                        Variant {
                            description: vec![],
                            name: "first-line"
                        },
                        Variant {
                            description: vec![],
                            name: "first-letter"
                        },
                        Variant {
                            description: vec![],
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
                        deprecated: false,
                        optional: true,
                        ty: Ty::Any,
                        name: "value"
                    },
                    Param {
                        description: vec!["One or more related nodes, if applicable."],
                        experimental: false,
                        deprecated: false,
                        optional: true,
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
                r#"    # Use 'Emulation.clearGeolocationOverride' instead
    redirect Emulation
"#
            )
            .unwrap(),
            (
                "",
                Redirect {
                    description: vec!["Use 'Emulation.clearGeolocationOverride' instead"],
                    to: "Emulation"
                }
            )
        )
    }
}
