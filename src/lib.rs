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
        Self::Type(String::from(""))
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

        let inner_enum = HashMap::from([
            (1, String::from("icmp")),
            (6, String::from("tcp")),
            (17, String::from("udp")),
        ]);

        let outer_enum = HashMap::from([(String::from("ip_protocol"), inner_enum)]);

        let expected = KaiTai {
            meta: Meta {
                id: String::from("animal_record"),
                endian: Some(Endian::Be),
            },
            seq: Vec::from([
                Seq {
                    id: String::from("uuid"),
                    size: Some(String::from("16")),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("name"),
                    size: Some(String::from("24")),
                    k_type: Some(KType::Type(String::from("str"))),
                    encoding: Some(String::from("UTF-8")),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("birth_year"),
                    k_type: Some(KType::Type(String::from("u2"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("weight"),
                    k_type: Some(KType::Type(String::from("f8"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("rating"),
                    k_type: Some(KType::Type(String::from("s4"))),
                    doc: Some(String::from("Rating, can be negative")),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("name"),
                    size: Some(String::from("16")),
                    k_type: Some(KType::Type(String::from("str"))),
                    encoding: Some(String::from("UTF-8")),
                    terminator: Some(String::from("0")),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("has_crc32"),
                    k_type: Some(KType::Type(String::from("u1"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("crc32"),
                    k_type: Some(KType::Type(String::from("u4"))),
                    k_if: Some(String::from("has_crc32 != 0")),
                    ..Seq::default()
                },
            ]),
            enums: Some(outer_enum),
            types: Some(HashMap::from([(
                String::from("str_with_len"),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: String::from("len"),
                            k_type: Some(KType::Type(String::from("u4"))),
                            ..Seq::default()
                        },
                        Seq {
                            id: String::from("value"),
                            k_type: Some(KType::Type(String::from("str"))),
                            encoding: Some(String::from("UTF-8")),
                            size: Some(String::from("len")),
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
                id: String::from("testing"),
                endian: None,
            },
            seq: Vec::from([
                Seq {
                    id: String::from("filenames"),
                    k_type: Some(KType::Type(String::from("filename"))),
                    repeat: Some(Repeat::EndOfStream),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("width"),
                    k_type: Some(KType::Type(String::from("u4"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("height"),
                    k_type: Some(KType::Type(String::from("u4"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("matrix"),
                    k_type: Some(KType::Type(String::from("f8"))),
                    repeat: Some(Repeat::Expression),
                    repeat_expr: Some(String::from("width * height")),
                    ..Seq::default()
                },
            ]),
            types: Some(HashMap::from([(
                String::from("filename"),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: String::from("name"),
                            k_type: Some(KType::Type(String::from("str"))),
                            size: Some(String::from("8")),
                            encoding: Some(String::from("ASCII")),
                            ..Seq::default()
                        },
                        Seq {
                            id: String::from("ext"),
                            k_type: Some(KType::Type(String::from("str"))),
                            size: Some(String::from("3")),
                            encoding: Some(String::from("ASCII")),
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
                id: String::from("testing"),
                endian: None,
            },
            seq: Vec::from([Seq {
                id: String::from("records"),
                k_type: Some(KType::Type(String::from("buffer_with_len"))),
                repeat: Some(Repeat::Until),
                repeat_until: Some(String::from("_.len == 0")),
                ..Seq::default()
            }]),
            types: Some(HashMap::from([(
                String::from("buffer_with_len"),
                SeqValue {
                    seq: Vec::from([
                        Seq {
                            id: String::from("len"),
                            k_type: Some(KType::Type(String::from("u1"))),
                            ..Seq::default()
                        },
                        Seq {
                            id: String::from("value"),
                            size: Some(String::from("len")),
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
                id: String::from("testing"),
                endian: None,
            },
            seq: Vec::from([
                Seq {
                    id: String::from("rec_type"),
                    k_type: Some(KType::Type(String::from("u1"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("len"),
                    k_type: Some(KType::Type(String::from("u4"))),
                    ..Seq::default()
                },
                Seq {
                    id: String::from("body"),
                    size: Some(String::from("len")),
                    k_type: Some(KType::Switch(KTypeSwitch {
                        switch_on: String::from("rec_type"),
                        cases: HashMap::from([
                            (Value::from(1), String::from("rec_type_1")),
                            (Value::from(2), String::from("rec_type_2")),
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
