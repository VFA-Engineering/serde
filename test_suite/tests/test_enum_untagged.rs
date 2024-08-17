#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

mod bytes;

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};
use std::collections::BTreeMap;

#[test]
fn complex() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Untagged {
        A { a: u8 },
        B { b: u8 },
        C,
        D(u8),
        E(String),
        F(u8, u8),
    }

    assert_tokens(
        &Untagged::A { a: 1 },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("a"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    assert_tokens(
        &Untagged::B { b: 2 },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("b"),
            Token::U8(2),
            Token::StructEnd,
        ],
    );

    // Serializes to unit, deserializes from either depending on format's
    // preference.
    assert_tokens(&Untagged::C, &[Token::Unit]);
    assert_de_tokens(&Untagged::C, &[Token::None]);

    assert_tokens(&Untagged::D(4), &[Token::U8(4)]);
    assert_tokens(&Untagged::E("e".to_owned()), &[Token::Str("e")]);

    assert_tokens(
        &Untagged::F(1, 2),
        &[
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(2),
            Token::TupleEnd,
        ],
    );

    assert_de_tokens_error::<Untagged>(
        &[Token::Tuple { len: 1 }, Token::U8(1), Token::TupleEnd],
        "data did not match any variant of untagged enum Untagged",
    );

    assert_de_tokens_error::<Untagged>(
        &[
            Token::Tuple { len: 3 },
            Token::U8(1),
            Token::U8(2),
            Token::U8(3),
            Token::TupleEnd,
        ],
        "data did not match any variant of untagged enum Untagged",
    );
}

#[test]
fn newtype_unit_and_empty_map() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Message {
        Unit(Unit),
        Map(BTreeMap<String, String>),
    }

    assert_tokens(
        &Message::Map(BTreeMap::new()),
        &[Token::Map { len: Some(0) }, Token::MapEnd],
    );
}

#[test]
fn newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct NewtypeStruct(u32);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum E {
        Newtype(NewtypeStruct),
        Null,
    }

    assert_tokens(
        &E::Newtype(NewtypeStruct(5)),
        &[
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::U32(5),
        ],
    );
}

#[test]
fn newtype_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Outer {
        Inner(Inner),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Inner {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { f: u8 },
    }

    assert_tokens(
        &Outer::Inner(Inner::Unit),
        &[Token::UnitVariant {
            name: "Inner",
            variant: "Unit",
        }],
    );

    assert_tokens(
        &Outer::Inner(Inner::Newtype(1)),
        &[
            Token::NewtypeVariant {
                name: "Inner",
                variant: "Newtype",
            },
            Token::U8(1),
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Tuple(1, 1)),
        &[
            Token::TupleVariant {
                name: "Inner",
                variant: "Tuple",
                len: 2,
            },
            Token::U8(1),
            Token::U8(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_tokens(
        &Outer::Inner(Inner::Struct { f: 1 }),
        &[
            Token::StructVariant {
                name: "Inner",
                variant: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn string_and_bytes() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Untagged {
        String {
            string: String,
        },
        Bytes {
            #[serde(with = "bytes")]
            bytes: Vec<u8>,
        },
    }

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::String {
            string: "\0".to_owned(),
        },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("string"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Str("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::String("\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Bytes(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::ByteBuf(b"\0"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &Untagged::Bytes { bytes: vec![0] },
        &[
            Token::Struct {
                name: "Untagged",
                len: 1,
            },
            Token::Str("bytes"),
            Token::Seq { len: Some(1) },
            Token::U8(0),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn contains_flatten() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Data {
        A {
            a: i32,
            #[serde(flatten)]
            flat: Flat,
        },
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Flat {
        b: i32,
    }

    let data = Data::A {
        a: 0,
        flat: Flat { b: 0 },
    };

    assert_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("a"),
            Token::I32(0),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
        ],
    );
}

#[test]
fn contains_flatten_with_integer_key() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Untagged {
        Variant {
            #[serde(flatten)]
            map: BTreeMap<u64, String>,
        },
    }

    assert_tokens(
        &Untagged::Variant {
            map: {
                let mut map = BTreeMap::new();
                map.insert(100, "BTreeMap".to_owned());
                map
            },
        },
        &[
            Token::Map { len: None },
            Token::U64(100),
            Token::Str("BTreeMap"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn expecting_message() {
    #[derive(Deserialize)]
    #[serde(untagged)]
    #[serde(expecting = "something strange...")]
    enum Enum {
        Untagged,
    }

    assert_de_tokens_error::<Enum>(&[Token::Str("Untagged")], "something strange...");
}
