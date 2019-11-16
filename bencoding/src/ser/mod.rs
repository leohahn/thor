use crate::error::{Error, Result};
use log::trace;
use serde::{ser, Serialize};
use std::io::Write;

mod utils;

#[derive(Debug, PartialEq)]
pub struct MapState {
    pub ordered_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    pub output: Vec<u8>,
    pub current_key: Vec<u8>,
}

impl MapState {
    fn new() -> MapState {
        MapState {
            ordered_pairs: vec![],
            output: vec![],
            current_key: vec![],
        }
    }
}

// pub struct Buffers {
//     pub main: Vec<u8>,
//     pub temp: Vec<u8>,
// }

pub struct Serializer {
    pub output: Vec<u8>,
    map_state: Option<MapState>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: Vec::new(),
        map_state: None,
    };
    value.serialize(&mut serializer)?;

    if let Some(map_state) = serializer.map_state {
        serializer.output.write(&map_state.output).unwrap();
    }
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
        trace!("Serializing bool: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        trace!("Serializing i8: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        trace!("Serializing i16: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        trace!("Serializing i32: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        // TODO: probably not that efficient
        trace!("Serializing i64: {}", v);
        Ok(utils::write_integer(&mut self.output, v))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        trace!("Serializing u8: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        trace!("Serializing u16: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        trace!("Serializing u32: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        trace!("Serializing u64: {}", v);
        Ok(utils::write_unsigned(&mut self.output, v))
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::Message("bencoding does not support f32".into()))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::Message("bencoding does not support f64".into()))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        trace!("Serializing char: {}", v);
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
        trace!("seq: main: {}", String::from_utf8_lossy(&self.output));
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
        assert!(self.map_state.is_none());
        self.map_state = Some(MapState::new());
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
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
        trace!("Serializing struct variant: {}", variant);
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
        value.serialize(&mut **self)?;

        if let Some(map_state) = self.map_state.take() {
            // A map/struct was serialized to map_state.output,
            // move it into the serialized value variable.
            self.output.write(&map_state.output).unwrap();
        }

        Ok(())
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
        use std::mem::swap;
        assert!(self.map_state.is_some());

        {
            let map_state = self.map_state.as_mut().expect("map_state should exist");
            map_state.current_key.clear();
            swap(&mut map_state.current_key, &mut self.output);
        }

        {
            trace!("Serializing key");
            key.serialize(&mut **self)?;
        }

        {
            let map_state = self.map_state.as_mut().expect("map_state should exist");
            swap(&mut map_state.current_key, &mut self.output);
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        use std::mem::swap;
        assert!(self.map_state.is_some());

        // the key was already serialized, now we need to serialize the value.
        // since the value could be a map/struct as well, we will backup the map_state.
        let backup_map_state = self.map_state.take();

        let mut serialized_value = vec![];
        {
            swap(&mut serialized_value, &mut self.output);

            trace!("Serializing value");
            value.serialize(&mut **self)?;

            if let Some(mut map_state) = self.map_state.take() {
                // A map/struct was serialized to map_state.output,
                // move it into the serialized value variable.
                swap(&mut serialized_value, &mut map_state.output);
            } else {
                swap(&mut serialized_value, &mut self.output);
            }
        }

        // Move backup map_state into the original position
        self.map_state = backup_map_state;

        let map_state: &mut MapState = self.map_state.as_mut().unwrap();

        if serialized_value.is_empty() {
            // If the value is None
            return Ok(());
        }

        trace!(
            "[map] adding key val: {} -> {}",
            String::from_utf8_lossy(&map_state.current_key),
            String::from_utf8_lossy(&serialized_value)
        );
        map_state
            .ordered_pairs
            .push((map_state.current_key.clone(), serialized_value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        assert!(self.map_state.is_some());
        let map_state: &mut MapState = self.map_state.as_mut().unwrap();

        write_dict_with_ordered_pairs(&mut map_state.ordered_pairs, &mut map_state.output)?;
        trace!(
            "[map] result: {}",
            String::from_utf8_lossy(&map_state.output),
        );
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
        use std::mem::swap;
        assert!(self.map_state.is_some());

        trace!("serialize_field({})", key);

        let mut serialized_key = vec![];
        {
            swap(&mut serialized_key, &mut self.output);

            trace!("Serializing key");
            key.serialize(&mut **self)?;

            swap(&mut serialized_key, &mut self.output);
        }

        // the key was already serialized, now we need to serialize the value.
        // since the value could be a map/struct as well, we will backup the map_state.
        let backup_map_state = self.map_state.take();

        let mut serialized_value = vec![];
        {
            swap(&mut serialized_value, &mut self.output);

            trace!("Serializing value");
            value.serialize(&mut **self)?;

            if let Some(mut map_state) = self.map_state.take() {
                // A map/struct was serialized to map_state.output,
                // move it into the serialized value variable.
                swap(&mut serialized_value, &mut map_state.output);
            } else {
                swap(&mut serialized_value, &mut self.output);
            }
        }

        // Move backup map_state into the original position
        self.map_state = backup_map_state;

        let map_state: &mut MapState = self.map_state.as_mut().unwrap();

        if serialized_value.is_empty() {
            // If the value is None
            return Ok(());
        }

        trace!(
            "[struct] adding key val: {} -> {}",
            String::from_utf8_lossy(&map_state.current_key),
            String::from_utf8_lossy(&serialized_value)
        );
        map_state
            .ordered_pairs
            .push((serialized_key, serialized_value));

        Ok(())
    }

    fn end(self) -> Result<()> {
        assert!(self.map_state.is_some());
        let map_state: &mut MapState = self.map_state.as_mut().unwrap();

        write_dict_with_ordered_pairs(&mut map_state.ordered_pairs, &mut map_state.output)?;
        trace!(
            "[struct] result: {}",
            String::from_utf8_lossy(&map_state.output),
        );
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
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        assert!(self.map_state.is_some());
        let map_state: &mut MapState = self.map_state.as_mut().unwrap();

        write_dict_with_ordered_pairs(&mut map_state.ordered_pairs, &mut map_state.output)?;
        map_state.output.write(b"e").unwrap();
        trace!(
            "[struct_variant] result: {}",
            String::from_utf8_lossy(&map_state.output),
        );
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
    ordered_pairs.clear();
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

#[test]
fn test_option_serialization() {
    use std::str;

    #[derive(Serialize)]
    struct S {
        string_field: String,
        other_field: Vec<String>,
        another_field: Option<String>,
    }

    {
        let s = S {
            another_field: None,
            string_field: "abcdef".to_owned(),
            other_field: vec![],
        };
        let bytes = to_bytes(&s).unwrap();
        let expected = "d11:other_fieldle12:string_field6:abcdefe";
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }
}

#[test]
fn test_nested_struct_serialization() {
    use std::str;

    #[derive(Serialize)]
    struct N {
        field1: u32,
        field2: i64,
    }

    #[derive(Serialize)]
    struct S {
        string_field: String,
        nested_field: N,
    }

    {
        let s = S {
            nested_field: N {
                field1: 20,
                field2: 64,
            },
            string_field: "abcdef".to_owned(),
        };
        let bytes = to_bytes(&s).unwrap();
        let expected = "d12:nested_fieldd6:field1i20e6:field2i64ee12:string_field6:abcdefe";
        assert_eq!(unsafe { str::from_utf8_unchecked(&bytes) }, expected);
    }
}
