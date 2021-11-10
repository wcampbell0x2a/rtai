//! rtai: Parser library for parsing KaiTai yaml files.
//!
//! Currently tested up to < 4.11 in https://doc.kaitai.io/user_guide.html

// TODO: Implement Types for k_type
// TODO: use serde_yaml::Value
// TODO: use serde_yaml::Mapping

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use serde_yaml::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct KaiTai {
    meta: Meta,
    seq: Vec<Seq>,
    enums: Option<HashMap<String, HashMap<u8, String>>>,
    types: Option<HashMap<String, SeqValue>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Meta {
    id: String,
    endian: Option<Endian>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Seq {
    id: String,

    size: Option<String>,

    // TODO: parse
    #[serde(rename = "type")]
    k_type: Option<KType>,

    encoding: Option<String>,

    doc: Option<String>,

    // TODO: parse
    contents: Option<String>,

    terminator: Option<String>,

    #[serde(rename = "enum")]
    k_enum: Option<String>,

    #[serde(rename = "if")]
    k_if: Option<String>,

    repeat: Option<Repeat>,

    #[serde(rename = "repeat-expr")]
    repeat_expr: Option<String>,

    #[serde(rename = "repeat-until")]
    repeat_until: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Endian {
    #[serde(rename = "le")]
    Le,

    #[serde(rename = "be")]
    Be,
}

impl Default for Endian {
    fn default() -> Self {
        Self::Le
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SeqValue {
    seq: Vec<Seq>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Repeat {
    #[serde(rename = "eos")]
    EndOfStream,

    #[serde(rename = "expr")]
    Expression,

    #[serde(rename = "until")]
    Until,
}

impl Default for Repeat {
    fn default() -> Self {
        Self::EndOfStream
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KType {
    Type(String),
    Switch(KTypeSwitch),
}

impl Default for KType {
    fn default() -> Self {
        Self::Type("".into())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct KTypeSwitch {
    #[serde(rename = "switch-on")]
    switch_on: String,
    cases: HashMap<Value, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let s = r#"meta:
  id: animal_record
  endian: be
seq:
  - id: uuid
    size: 16
  - id: name
    type: str
    size: 24
    encoding: UTF-8
  - id: birth_year
    type: u2
  - id: weight
    type: f8
  - id: rating
    type: s4
    doc: Rating, can be negative
  - id: name
    type: str
    size: 16
    terminator: 0
    encoding: UTF-8
  - id: has_crc32
    type: u1
  - id: crc32
    type: u4
    if: has_crc32 != 0
enums:
  ip_protocol:
    1: icmp
    6: tcp
    17: udp
types:
  str_with_len:
    seq:
      - id: len
        type: u4
      - id: value
        type: str
        encoding: UTF-8
        size: len
"#;
        let kaitai: KaiTai = serde_yaml::from_str(&s).unwrap();

        let inner_enum = HashMap::from([(1, "icmp".into()), (6, "tcp".into()), (17, "udp".into())]);

        let outer_enum = HashMap::from([("ip_protocol".into(), inner_enum)]);

        let expected = KaiTai {
            meta: Meta {
                id: "animal_record".into(),
                endian: Some(Endian::Be),
            },
            seq: Vec::from([
                Seq {
                    id: "uuid".into(),
                    size: Some("16".into()),
                    ..Seq::default()
                },
                Seq {
                    id: "name".into(),
                    size: Some("24".into()),
                    k_type: Some(KType::Type("str".into())),
                    encoding: Some("UTF-8".into()),
                    ..Seq::default()
                },
                Seq {
                    id: "birth_year".into(),
                    k_type: Some(KType::Type("u2".into())),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("weight"),
                    k_type: Some(KType::Type("f8".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "rating".into(),
                    k_type: Some(KType::Type("s4".into())),
                    doc: Some("Rating, can be negative".into()),
                    ..Seq::default()
                },
                Seq {
                    id: "name".into(),
                    size: Some("16".into()),
                    k_type: Some(KType::Type("str".into())),
                    encoding: Some("UTF-8".into()),
                    terminator: Some("0".into()),
                    ..Seq::default()
                },
                Seq {
                    id: "has_crc32".into(),
                    k_type: Some(KType::Type("u1".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "crc32".into(),
                    k_type: Some(KType::Type("u4".into())),
                    k_if: Some("has_crc32 != 0".into()),
                    ..Seq::default()
                },
            ]),
            enums: Some(outer_enum),
            types: Some(HashMap::from([(
                "str_with_len".into(),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: "len".into(),
                            k_type: Some(KType::Type("u4".into())),
                            ..Seq::default()
                        },
                        Seq {
                            id: "value".into(),
                            k_type: Some(KType::Type("str".into())),
                            encoding: Some("UTF-8".into()),
                            size: Some("len".into()),
                            ..Seq::default()
                        },
                    ]),
                },
            )])),
        };

        assert_eq!(expected, kaitai);
    }

    #[test]
    fn test_eos_parsing() {
        let s = r#"meta:
  id: testing
seq:
  - id: filenames
    type: filename
    repeat: eos
  - id: width
    type: u4
  - id: height
    type: u4
  - id: matrix
    type: f8
    repeat: expr
    repeat-expr: width * height
types:
  filename:
    seq:
      - id: name
        type: str
        size: 8
        encoding: ASCII
      - id: ext
        type: str
        size: 3
        encoding: ASCII
"#;
        let kaitai: KaiTai = serde_yaml::from_str(&s).unwrap();

        let expected = KaiTai {
            meta: Meta {
                id: "testing".into(),
                endian: None,
            },
            seq: Vec::from([
                Seq {
                    id: "filenames".into(),
                    k_type: Some(KType::Type("filename".into())),
                    repeat: Some(Repeat::EndOfStream),
                    ..Seq::default()
                },
                Seq {
                    id: "width".into(),
                    k_type: Some(KType::Type("u4".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "height".into(),
                    k_type: Some(KType::Type("u4".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "matrix".into(),
                    k_type: Some(KType::Type("f8".into())),
                    repeat: Some(Repeat::Expression),
                    repeat_expr: Some("width * height".into()),
                    ..Seq::default()
                },
            ]),
            types: Some(HashMap::from([(
                "filename".into(),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: "name".into(),
                            k_type: Some(KType::Type("str".into())),
                            size: Some("8".into()),
                            encoding: Some("ASCII".into()),
                            ..Seq::default()
                        },
                        Seq {
                            id: "ext".into(),
                            k_type: Some(KType::Type("str".into())),
                            size: Some("3".into()),
                            encoding: Some("ASCII".into()),
                            ..Seq::default()
                        },
                    ]),
                },
            )])),
            ..KaiTai::default()
        };

        assert_eq!(expected, kaitai);
    }

    #[test]
    fn test_repeat_until_condition() {
        let s = r#"meta:
  id: testing
seq:
  - id: records
    type: buffer_with_len
    repeat: until
    repeat-until: _.len == 0
types:
  buffer_with_len:
    seq:
      - id: len
        type: u1
      - id: value
        size: len
"#;
        let kaitai: KaiTai = serde_yaml::from_str(&s).unwrap();

        let expected = KaiTai {
            meta: Meta {
                id: "testing".into(),
                endian: None,
            },
            seq: Vec::from([Seq {
                id: "records".into(),
                k_type: Some(KType::Type("buffer_with_len".into())),
                repeat: Some(Repeat::Until),
                repeat_until: Some("_.len == 0".into()),
                ..Seq::default()
            }]),
            types: Some(HashMap::from([(
                "buffer_with_len".into(),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: "len".into(),
                            k_type: Some(KType::Type("u1".into())),
                            ..Seq::default()
                        },
                        Seq {
                            id: "value".into(),
                            size: Some("len".into()),
                            ..Seq::default()
                        },
                    ]),
                },
            )])),
            ..KaiTai::default()
        };

        assert_eq!(expected, kaitai);
    }

    #[test]
    fn test_4_11_tlv_impl() {
        let s = r#"meta:
    id: testing
seq:
  - id: rec_type
    type: u1
  - id: len
    type: u4
  - id: body
    size: len
    type:
      switch-on: rec_type
      cases:
        1: rec_type_1
        2: rec_type_2
"#;
        let kaitai: KaiTai = serde_yaml::from_str(&s).unwrap();

        let expected = KaiTai {
            meta: Meta {
                id: "testing".into(),
                endian: None,
            },
            seq: Vec::from([
                Seq {
                    id: "rec_type".into(),
                    k_type: Some(KType::Type("u1".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "len".into(),
                    k_type: Some(KType::Type("u4".into())),
                    ..Seq::default()
                },
                Seq {
                    id: "body".into(),
                    size: Some("len".into()),
                    k_type: Some(KType::Switch(KTypeSwitch {
                        switch_on: "rec_type".into(),
                        cases: HashMap::from([
                            (Value::from(1), "rec_type_1".into()),
                            (Value::from(2), "rec_type_2".into()),
                        ]),
                        ..KTypeSwitch::default()
                    })),
                    ..Seq::default()
                },
            ]),
            ..KaiTai::default()
        };

        assert_eq!(expected, kaitai);
    }
}
