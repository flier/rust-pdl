use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

use crate::*;

pub fn serialize_usize<S>(n: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&n.to_string())
}

pub fn serialize_description<S>(description: &[&str], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&description.join(" "))
}

pub fn serialize_enum<S>(variants: &[Variant], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(variants.len()))?;

    for variant in variants {
        seq.serialize_element(variant.name)?;
    }

    seq.end()
}

pub fn serialize_redirect<S>(redirect: &Option<Redirect>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(redirect) = redirect {
        serializer.serialize_str(&redirect.to)
    } else {
        serializer.serialize_none()
    }
}

pub fn is_false(v: &bool) -> bool {
    !*v
}

impl Protocol<'_> {
    /// Serialize the `Protocol` data structure as a String of JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Serialize the `Protocol` data structure as a pretty-printed String of JSON.
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl Serialize for Ty<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        match self {
            Ty::Integer => {
                map.serialize_entry("type", "integer")?;
            }
            Ty::Number => {
                map.serialize_entry("type", "number")?;
            }
            Ty::Boolean => {
                map.serialize_entry("type", "boolean")?;
            }
            Ty::String => {
                map.serialize_entry("type", "string")?;
            }
            Ty::Object => {
                map.serialize_entry("type", "object")?;
            }
            Ty::Any => {
                map.serialize_entry("type", "any")?;
            }
            Ty::Enum(variants) => {
                map.serialize_entry("type", "string")?;
                map.serialize_entry(
                    "enum",
                    &variants
                        .iter()
                        .map(|variant| variant.name)
                        .collect::<Vec<_>>(),
                )?;
            }
            Ty::ArrayOf(ty) => {
                map.serialize_entry("type", "array")?;
                map.serialize_entry("items", &ty)?;
            }
            Ty::Ref(id) => {
                map.serialize_entry("$ref", &id)?;
            }
        }

        map.end()
    }
}
