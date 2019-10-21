use crate::error::{Error, Result};
use log::trace;
use serde::{ser, Serialize};
use std::io::Write;

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: vec![] };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        trace!("Serializing bool");
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        trace!("Serializing i8");
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        trace!("Serializing i16");
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        trace!("Serializing i32");
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        // TODO: probably not that efficient
        trace!("Serializing i64");
        Ok(write!(&mut self.output, "i{}e", v).unwrap())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        trace!("Serializing u8");
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        trace!("Serializing u16");
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        trace!("Serializing u32");
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        trace!("Serializing u64");
        // TODO: probably not that efficient
        Ok(write!(&mut self.output, "i{}e", v).unwrap())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::Message("bencoding does not support f32".into()))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::Message("bencoding does not support f64".into()))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        trace!("Serializing char");
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        trace!("Serializing str: {}", v);
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        trace!("Serializing bytes");
        write!(&mut self.output, "{}:", v.len()).unwrap();
        self.output.write(v).unwrap();
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        // there is no representation of None in bencoding, so we just ignore it
        trace!("Serializing none");
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing some");
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        // there is no representation of Unit in bencoding, so we just ignore it
        trace!("Serializing unit");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        trace!("Serializing unit struct");
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        trace!("Serializing unit variant");
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing new type struct");
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing new type variant");
        use ser::SerializeMap;
        let mut map: Self::SerializeMap = self.serialize_map(None)?;
        map.serialize_key(variant)?;
        map.serialize_value(value)?;
        map.end()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        trace!("Serializing seq");
        self.output.write(b"l").unwrap();
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        trace!("Serializing tuple");
        self.output.write(b"l").unwrap();
        Ok(self)
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        trace!("Serialize tuple struct");
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        trace!("Serializing tuple variant");
        self.output.write(b"d").unwrap();
        variant.serialize(&mut *self)?;
        self.output.write(b"l").unwrap();
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        trace!("Serializing map");
        self.output.write(b"d").unwrap();
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        trace!("Serializing struct");
        self.serialize_map(Some(len))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        trace!("Serializing struct variant");
        self.output.write(b"d").unwrap();
        variant.serialize(&mut *self)?;
        self.output.write(b"d").unwrap();
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"ee").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing key");
        key.serialize(&mut **self)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing value");
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write(b"ee").unwrap();
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct_serialization() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["20", "40"],
    };

    let expected = "d3:inti1e3:seql2:202:40ee";
    assert_eq!(to_bytes(&test).unwrap(), expected.as_bytes());
}

#[test]
fn test_enum_serialization() {
    use std::str;

    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    {
        let u = E::Unit;
        let bytes = to_bytes(&u).unwrap();
        let expected = "4:Unit";
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }

    {
        let n = E::Newtype(1);
        let bytes = to_bytes(&n).unwrap();
        let expected = "d7:Newtypei1ee";
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }

    {
        let t = E::Tuple(1, 2);
        let bytes = to_bytes(&t).unwrap();
        let expected = "d5:Tupleli1ei2eee";
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }

    {
        let s = E::Struct { a: 1 };
        let expected = "d6:Structd1:ai1eee";
        let bytes = to_bytes(&s).unwrap();
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }
}
