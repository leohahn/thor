use crate::error::{Error, Result};
use log::trace;
use serde::{ser, Serialize};
use std::io::Write;

mod utils;

pub struct Serializer {
    pub output: Vec<u8>,
    ordered_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    current_key: Vec<u8>,
    original_output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: vec![],
        ordered_pairs: vec![],
        current_key: vec![],
        original_output: vec![],
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl Serializer {
    fn switch_to_temp_buffer(&mut self) {
        // We swap the two buffers. This is necessary in order not to mix the current
        // output (the final one) with a temporary buffer used for serializing a map,
        // for example.
        self.original_output.clear();
        std::mem::swap(&mut self.original_output, &mut self.output);
    }

    fn switch_to_original_buffer(&mut self) {
        // Move the original buffer back to the original variable self.output.
        self.output.clear();
        std::mem::swap(&mut self.original_output, &mut self.output);
    }
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
        Ok(utils::write_integer(&mut self.output, v))
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
        Ok(utils::write_unsigned(&mut self.output, v))
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
        println!("Serializing new type variant");
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
        self.switch_to_temp_buffer();
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
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        trace!("Serializing struct variant, {}", variant);
        println!("Serializing struct variant, {}", variant);
        self.output.write(b"d").unwrap();
        variant.serialize(&mut *self)?;
        self.serialize_map(Some(len))
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

////////////////////////////////////////////////////////////////////
/// Map Serializer and similar ones
////////////////////////////////////////////////////////////////////
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing key");
        key.serialize(&mut **self)?;

        self.current_key = vec![];
        std::mem::swap(&mut self.current_key, &mut self.output);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing value");
        value.serialize(&mut **self)?;

        if self.output.len() == 0 {
            // If the value is None
            self.current_key = vec![];
            return Ok(());
        }

        assert!(self.current_key.len() > 0);

        let mut key = vec![];
        std::mem::swap(&mut self.current_key, &mut key);
        let mut val = vec![];
        std::mem::swap(&mut self.output, &mut val);

        self.ordered_pairs.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.switch_to_original_buffer();
        write_dict_with_ordered_pairs(&mut self.ordered_pairs, &mut self.output)?;
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        let mut serialized_key = vec![];
        std::mem::swap(&mut serialized_key, &mut self.output);

        value.serialize(&mut **self)?;
        let mut serialized_value = vec![];
        std::mem::swap(&mut serialized_value, &mut self.output);

        self.ordered_pairs.push((serialized_key, serialized_value));
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.switch_to_original_buffer();
        write_dict_with_ordered_pairs(&mut self.ordered_pairs, &mut self.output)?;
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        let mut serialized_key = vec![];
        std::mem::swap(&mut serialized_key, &mut self.output);

        value.serialize(&mut **self)?;
        let mut serialized_value = vec![];
        std::mem::swap(&mut serialized_value, &mut self.output);

        self.ordered_pairs.push((serialized_key, serialized_value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.switch_to_original_buffer();
        write_dict_with_ordered_pairs(&mut self.ordered_pairs, &mut self.output)?;
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

fn write_dict_with_ordered_pairs(
    ordered_pairs: &mut Vec<(Vec<u8>, Vec<u8>)>,
    output: &mut Vec<u8>,
) -> Result<()> {
    ordered_pairs.sort_by_cached_key(|k: &(Vec<u8>, Vec<u8>)| {
        let key: &[u8] = k.0.as_ref();
        let index = key
            .iter()
            .position(|b| *b == b':')
            .expect("should have a :");
        let (_, new_key) = key.split_at(index + 1);
        new_key.to_owned()
    });

    output.write(b"d").unwrap();
    for (key, val) in ordered_pairs.iter() {
        trace!("writing key {}", unsafe {
            std::str::from_utf8_unchecked(key)
        });
        output.write(&key).unwrap();
        output.write(&val).unwrap();
    }
    output.write(b"e").unwrap();
    Ok(())
}

//////////////////////////////////////////////////////////////////////
/// Tests
//////////////////////////////////////////////////////////////////////

#[test]
fn test_struct_serialization() {
    #[derive(Serialize)]
    struct Test {
        seq: Vec<&'static str>,
        int: u32,
    }

    let test = Test {
        seq: vec!["20", "40"],
        int: 1,
    };

    let expected = "d3:inti1e3:seql2:202:40ee";
    assert_eq!(to_bytes(&test).unwrap(), expected.as_bytes());
}

#[test]
fn test_map_serialization() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("my_key", 20);
    map.insert("other_key", 1000);
    map.insert("abc", 501);

    let expected = "d3:abci501e6:my_keyi20e9:other_keyi1000ee";
    assert_eq!(to_bytes(&map).unwrap(), expected.as_bytes());
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
        StructSorted { uiui: String, abc: u32, ppp: u8 },
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

    {
        let s = E::StructSorted {
            uiui: "temp_string".to_string(),
            abc: 1024,
            ppp: 1,
        };
        let expected = "d12:StructSortedd3:abci1024e3:pppi1e4:uiui11:temp_stringee";
        let bytes = to_bytes(&s).unwrap();
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }
}
