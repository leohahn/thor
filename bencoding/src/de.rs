use log::trace;
use std::ops::{AddAssign, MulAssign, Neg};
use std::str;

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    // This vector starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_bytes`.
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    // Look at the first character in the input without consuming it.
    fn peek_byte(&mut self) -> Result<u8> {
        if self.input.len() == 0 {
            Err(Error::Eof)
        } else {
            Ok(self.input[0])
        }
    }

    fn peek_two_bytes(&mut self) -> Result<(u8, u8)> {
        if self.input.len() < 2 {
            Err(Error::Eof)
        } else {
            Ok((self.input[0], self.input[1]))
        }
    }

    // Consume the first byte in the input.
    fn next_byte(&mut self) -> Result<u8> {
        let ch = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(ch)
    }

    // Parse a group of decimal digits as an unsigned integer of type T.
    //
    // This implementation is a bit too lenient, for example `001` is not
    // allowed in JSON. Also the various arithmetic operations can overflow and
    // panic or return bogus data. But it is good enough for example code!
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_byte()? {
            ch @ b'0'..=b'9' => T::from(ch - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input.first() {
                Some(ch @ b'0'..=b'9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch - b'0');
                }
                _ => {
                    return Ok(int);
                }
            }
        }
    }

    // Parse a possible minus sign followed by a group of decimal digits as a
    // signed integer of type T.
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        // Optional minus sign, delegate to `parse_unsigned`, negate if negative.
        let is_negative = match self.peek_byte()? {
            b'-' => {
                self.next_byte().unwrap(); // this should not fail since we already peeked the byte
                true
            }
            _ => false,
        };

        let mut int = match self.next_byte()? {
            ch @ b'0'..=b'9' => T::from((ch as i8) - (b'0' as i8)),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input.first() {
                Some(ch @ b'0'..=b'9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from((*ch as i8) - (b'0' as i8));
                }
                _ => {
                    if is_negative {
                        return Ok(-int);
                    } else {
                        return Ok(int);
                    }
                }
            }
        }
    }

    fn parse_bencoding_num_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8> + std::fmt::Display,
    {
        if self.next_byte()? == b'i' {
            let num = self.parse_unsigned()?;
            if self.next_byte()? == b'e' {
                Ok(num)
            } else {
                Err(Error::ExpectedIntegerEnd)
            }
        } else {
            Err(Error::ExpectedInteger)
        }
    }

    fn parse_bencoding_num_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        if self.next_byte()? == b'i' {
            let num = self.parse_signed()?;
            if self.next_byte()? == b'e' {
                Ok(num)
            } else {
                Err(Error::ExpectedIntegerEnd)
            }
        } else {
            Err(Error::ExpectedInteger)
        }
    }

    fn parse_byte_string(&mut self) -> Result<&'de [u8]> {
        let len: usize = self.parse_unsigned()?;

        if self.next_byte()? != b':' {
            return Err(Error::ExpectedByteString);
        }

        if self.input.len() < len {
            return Err(Error::Eof);
        }

        let s = &self.input[..len];
        self.input = &self.input[len..];
        Ok(s)
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        let byte_string = self.parse_byte_string()?;
        if let Ok(string) = str::from_utf8(byte_string) {
            Ok(string)
        } else {
            Err(Error::ExpectedString)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_byte()? {
            b'0'..=b'9' => self.deserialize_str(visitor),
            b'i' => match self.peek_two_bytes()? {
                (_, b'-') => self.deserialize_i64(visitor),
                _ => self.deserialize_u64(visitor),
            },
            b'l' => self.deserialize_seq(visitor),
            b'd' => self.deserialize_map(visitor),
            _ => Err(Error::Syntax),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing bool");
        let num = self.parse_bencoding_num_unsigned::<u32>()?;
        visitor.visit_bool(num != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing i8");
        let num = self.parse_bencoding_num_signed()?;
        visitor.visit_i8(num)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing i16");
        let num = self.parse_bencoding_num_signed()?;
        visitor.visit_i16(num)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing i32");
        let num = self.parse_bencoding_num_signed()?;
        visitor.visit_i32(num)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing i64");
        let num = self.parse_bencoding_num_signed()?;
        visitor.visit_i64(num)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing u8");
        let num = self.parse_bencoding_num_unsigned()?;
        visitor.visit_u8(num)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing u16");
        let num = self.parse_bencoding_num_unsigned()?;
        visitor.visit_u16(num)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing u32");
        let num = self.parse_bencoding_num_unsigned()?;
        visitor.visit_u32(num)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing u64");
        let num = self.parse_bencoding_num_unsigned()?;
        visitor.visit_u64(num)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing char");
        // Parse a string, check that it is one character, call `visit_char`.
        let byte_string = self.parse_byte_string()?;
        let string = std::str::from_utf8(byte_string).map_err(|_e| Error::ExpectedChar)?;

        let mut chars = string.chars();
        let first_char = chars.next();
        let second_char = chars.next();

        if first_char.is_none() {
            return Err(Error::ExpectedChar);
        }

        if second_char.is_some() {
            return Err(Error::ExpectedChar);
        }

        // we can safely call unwrap here
        visitor.visit_char(first_char.unwrap())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing str");
        let byte_string = self.parse_byte_string()?;
        if let Ok(string) = str::from_utf8(byte_string) {
            trace!("    str = {}", string);
            visitor.visit_borrowed_str(string)
        } else {
            Err(Error::ExpectedString)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing string");
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing bytes");
        let byte_string = self.parse_byte_string()?;
        visitor.visit_bytes(byte_string)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing byte buf");
        let byte_string = self.parse_byte_string()?;
        visitor.visit_bytes(byte_string)
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing option");
        visitor.visit_some(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing newtype struct");
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing seq");
        // Parse the opening bracket of the sequence.
        if self.next_byte()? == b'l' {
            // Give the visitor access to each element of the sequence.
            let value = visitor.visit_seq(Values::new(&mut self))?;
            // Parse the closing bracket of the sequence.
            if self.next_byte()? == b'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedList)
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing tuple");
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing tuple struct");
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing map");
        // Parse the opening brace of the map.
        if self.next_byte()? == b'd' {
            // Give the visitor access to each entry of the map.
            let value = visitor.visit_map(Values::new(&mut self))?;
            // Parse the closing brace of the map.
            if self.next_byte()? == b'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing struct {}", name);
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing enum {}", name);
        match self.peek_byte()? {
            b'0'..=b'9' => visitor.visit_enum(self.parse_string()?.into_deserializer()),
            b'd' => {
                self.next_byte().unwrap(); // safe to call unwrap here
                let value = visitor.visit_enum(Enum::new(self))?;
                if self.next_byte()? == b'e' {
                    Ok(value)
                } else {
                    Err(Error::ExpectedMapEnd)
                }
            }
            _ => Err(Error::ExpectedEnum),
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing identifier");
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("Deserializing ignored any");
        self.deserialize_any(visitor)
    }
}

struct Values<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Values<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Values { de }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for Values<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek_byte()? == b'e' {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for Values<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        match self.de.peek_byte()? {
            b'e' => Ok(None),
            b'0'..=b'9' => seed.deserialize(&mut *self.de).map(Some),
            _ => Err(Error::ExpectedString),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `d` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct_deserialization() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    let j = b"d3:inti1e3:seql1:a1:bee";
    let expected = Test {
        int: 1,
        seq: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(expected, from_bytes(j).unwrap());
}

#[test]
fn test_enum_deserialization() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = b"4:Unit";
    let expected = E::Unit;
    let e = from_bytes(j).unwrap();
    assert_eq!(expected, e);

    let j = b"d7:Newtypei1ee";
    let expected = E::Newtype(1);
    let e = from_bytes(j).unwrap();
    assert_eq!(expected, e);

    let j = b"d5:Tupleli1ei2eee";
    let expected = E::Tuple(1, 2);
    let e = from_bytes(j).unwrap();
    assert_eq!(expected, e);

    let j = b"d6:Structd1:ai1eee";
    let expected = E::Struct { a: 1 };
    let e = from_bytes(j).unwrap();
    assert_eq!(expected, e);
}
