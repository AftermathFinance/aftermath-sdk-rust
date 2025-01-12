use std::fmt::Display;

use base64::engine::GeneralPurpose;
use base64::Engine;
use bson::spec::BinarySubtype;
use bson::{Array, Binary, Bson, Decimal128, Document};
use serde::ser::{
    self,
    Serialize,
    SerializeMap,
    SerializeSeq,
    SerializeStruct,
    SerializeStructVariant,
    SerializeTuple,
    SerializeTupleStruct,
    SerializeTupleVariant,
};

pub fn to_move_bson<T>(value: &T) -> Result<Bson>
where
    T: ?Sized + Serialize,
{
    value.serialize(MoveBsonSerializer)
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Move types don't use signed integers")]
    SignedInteger,
    #[error("Move types don't use floating point numbers")]
    FloatingPoint,
    /// A key could not be serialized to a BSON string.
    #[error("Invalid map key type: {0}")]
    InvalidDocumentKey(Bson),
    #[error("Custom error: {0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// Serde Serializer for Sui Objects
///
/// Based on the SuiJSON specification:
/// <https://docs.sui.io/references/sui-api>
pub struct MoveBsonSerializer;

impl ser::Serializer for MoveBsonSerializer {
    type Ok = Bson;
    type Error = Error;

    type SerializeSeq = ArraySerializer;
    type SerializeTuple = TupleSerializer;
    type SerializeTupleStruct = TupleStructSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Bson> {
        Ok(Bson::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> Result<Bson> {
        Err(Error::SignedInteger)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> Result<Bson> {
        Err(Error::SignedInteger)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> Result<Bson> {
        Err(Error::SignedInteger)
    }

    #[inline]
    fn serialize_i64(self, _: i64) -> Result<Bson> {
        Err(Error::SignedInteger)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    fn serialize_u128(self, value: u128) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> Result<Bson> {
        Err(Error::FloatingPoint)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> Result<Bson> {
        Err(Error::FloatingPoint)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Bson> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Bson> {
        Ok(Bson::String(value.to_string()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Bson> {
        Ok(Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: value.to_vec(),
        }))
    }

    #[inline]
    fn serialize_none(self) -> Result<Bson> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(self, value: &V) -> Result<Bson>
    where
        V: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Bson> {
        Ok(Bson::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Bson> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Bson> {
        Ok(Bson::String(variant.to_string()))
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Bson>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Bson>
    where
        T: ?Sized + Serialize,
    {
        let mut newtype_variant = Document::new();
        newtype_variant.insert(variant, to_move_bson(value)?);
        Ok(newtype_variant.into())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(ArraySerializer {
            inner: Array::with_capacity(len.unwrap_or(0)),
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleSerializer {
            inner: Array::with_capacity(len),
        })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleStructSerializer {
            inner: Array::with_capacity(len),
        })
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantSerializer {
            inner: Array::with_capacity(len),
            name: variant,
        })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            inner: Document::new(),
            next_key: None,
        })
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructSerializer {
            inner: Document::new(),
        })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer {
            name: variant,
            inner: Document::new(),
        })
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[doc(hidden)]
pub struct ArraySerializer {
    inner: Array,
}

impl SerializeSeq for ArraySerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.push(to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        Ok(Bson::Array(self.inner))
    }
}

#[doc(hidden)]
pub struct TupleSerializer {
    inner: Array,
}

impl SerializeTuple for TupleSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.push(to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        Ok(Bson::Array(self.inner))
    }
}

#[doc(hidden)]
pub struct TupleStructSerializer {
    inner: Array,
}

impl SerializeTupleStruct for TupleStructSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.push(to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        Ok(Bson::Array(self.inner))
    }
}

#[doc(hidden)]
pub struct TupleVariantSerializer {
    inner: Array,
    name: &'static str,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.push(to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        let mut tuple_variant = Document::new();
        tuple_variant.insert(self.name, self.inner);
        Ok(tuple_variant.into())
    }
}

#[doc(hidden)]
pub struct MapSerializer {
    inner: Document,
    next_key: Option<String>,
}

impl SerializeMap for MapSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.next_key = match to_move_bson(&key)? {
            Bson::String(s) => Some(s),
            other => return Err(Error::InvalidDocumentKey(other)),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.next_key.take().unwrap_or_default();
        self.inner.insert(key, to_move_bson(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        Ok(from_extended_document(self.inner))
    }
}

#[doc(hidden)]
pub struct StructSerializer {
    inner: Document,
}

impl SerializeStruct for StructSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.inner.insert(key, to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        Ok(from_extended_document(self.inner))
    }
}

#[doc(hidden)]
pub struct StructVariantSerializer {
    inner: Document,
    name: &'static str,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Bson;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.inner.insert(key, to_move_bson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Bson> {
        let var = from_extended_document(self.inner);

        let mut struct_variant = Document::new();
        struct_variant.insert(self.name, var);

        Ok(Bson::Document(struct_variant))
    }
}

#[doc(hidden)]
pub fn from_extended_document(doc: Document) -> Bson {
    if doc.len() > 2 {
        return Bson::Document(doc);
    }

    let mut keys: Vec<_> = doc.keys().map(|s| s.as_str()).collect();
    keys.sort_unstable();

    match keys.as_slice() {
        ["$symbol"] => {
            if let Ok(symbol) = doc.get_str("$symbol") {
                return Bson::Symbol(symbol.into());
            }
        }

        ["$numberInt"] => {
            if let Ok(i) = doc.get_str("$numberInt") {
                if let Ok(i) = i.parse() {
                    return Bson::Int32(i);
                }
            }
        }

        ["$numberLong"] => {
            if let Ok(i) = doc.get_str("$numberLong") {
                if let Ok(i) = i.parse() {
                    return Bson::Int64(i);
                }
            }
        }

        ["$numberDouble"] => match doc.get_str("$numberDouble") {
            Ok("Infinity") => return Bson::Double(f64::INFINITY),
            Ok("-Infinity") => return Bson::Double(f64::NEG_INFINITY),
            Ok("NaN") => return Bson::Double(f64::NAN),
            Ok(other) => {
                if let Ok(d) = other.parse() {
                    return Bson::Double(d);
                }
            }
            _ => {}
        },

        ["$numberDecimal"] => {
            if let Ok(d) = doc.get_str("$numberDecimal") {
                if let Ok(d) = d.parse() {
                    return Bson::Decimal128(d);
                }
            }
        }

        ["$numberDecimalBytes"] => {
            if let Ok(bytes) = doc.get_binary_generic("$numberDecimalBytes") {
                if let Ok(b) = bytes.clone().try_into() {
                    return Bson::Decimal128(Decimal128::from_bytes(b));
                }
            }
        }

        ["$binary"] => {
            if let Some(binary) = binary_from_extended_doc(&doc) {
                return Bson::Binary(binary);
            }
        }

        ["$minKey"] => {
            let min_key = doc.get("$minKey");

            if min_key == Some(&Bson::Int32(1)) || min_key == Some(&Bson::Int64(1)) {
                return Bson::MinKey;
            }
        }

        ["$maxKey"] => {
            let max_key = doc.get("$maxKey");

            if max_key == Some(&Bson::Int32(1)) || max_key == Some(&Bson::Int64(1)) {
                return Bson::MaxKey;
            }
        }

        ["$undefined"] => {
            if doc.get("$undefined") == Some(&Bson::Boolean(true)) {
                return Bson::Undefined;
            }
        }

        _ => {}
    };

    Bson::Document(
        doc.into_iter()
            .map(|(k, v)| {
                let v = match v {
                    Bson::Document(v) => from_extended_document(v),
                    other => other,
                };

                (k, v)
            })
            .collect(),
    )
}

#[doc(hidden)]
pub const BASE64_DEFAULT_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::NO_PAD,
);

#[doc(hidden)]
pub fn binary_from_extended_doc(doc: &Document) -> Option<Binary> {
    let binary_doc = doc.get_document("$binary").ok()?;

    if let Ok(bytes) = binary_doc.get_str("base64") {
        let bytes = BASE64_DEFAULT_ENGINE.decode(bytes).ok()?;
        let subtype = binary_doc.get_str("subType").ok()?;
        let subtype = hex::decode(subtype).ok()?;
        if subtype.len() == 1 {
            Some(Binary {
                bytes,
                subtype: subtype[0].into(),
            })
        } else {
            None
        }
    } else {
        // in non-human-readable mode, RawBinary will serialize as
        // { "$binary": { "bytes": <bytes>, "subType": <i32> } };
        let binary = binary_doc.get_binary_generic("bytes").ok()?;
        let subtype = binary_doc.get_i32("subType").ok()?;

        Some(Binary {
            bytes: binary.clone(),
            subtype: u8::try_from(subtype).ok()?.into(),
        })
    }
}
