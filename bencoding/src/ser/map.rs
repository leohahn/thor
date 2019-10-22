use super::Serializer;
use crate::error::{Error, Result};
use log::trace;
use serde::{ser, Serialize};
use std::io::Write;
use std::str;

/////////////////////////////////////////////////////////////////////////////////////
//////////////// Map serializer /////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////
pub(crate) struct MapSerializer {
    se: Serializer,
    // TODO: remove BTreeMap here, since it will sort by the literal bencoding string, which
    // starts with a number prefix. The correct way is to use a vector here and sort it later.
    ordered_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    current_key: Vec<u8>,
    output: Vec<u8>,
}

impl MapSerializer {
    pub fn new() -> MapSerializer {
        MapSerializer {
            se: Serializer { output: vec![] },
            ordered_pairs: vec![],
            current_key: vec![],
            output: vec![],
        }
    }
}

impl<'a> ser::SerializeMap for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing key");
        key.serialize(&mut self.se)?;
        self.current_key = self.se.output;
        self.se.output = vec![];
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        trace!("Serializing value");
        value.serialize(&mut self.se)?;

        if self.se.output.len() == 0 {
            // If the value is None
            self.current_key = vec![];
            return Ok(());
        }

        assert!(self.current_key.len() > 0);

        let key = vec![];
        std::mem::swap(&mut self.current_key, &mut key);
        let val = vec![];
        std::mem::swap(&mut self.se.output, &mut val);

        self.ordered_pairs.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.ordered_pairs
            .sort_by_cached_key(|k: &(Vec<u8>, Vec<u8>)| {
                let key: &[u8] = k.0.as_ref();
                let index = key
                    .iter()
                    .position(|b| *b == b':')
                    .expect("should have a :");
                let (_, new_key) = key.split_at(index + 1);
                new_key.to_owned()
            });

        self.output.write(b"d").unwrap();
        for (key, val) in self.ordered_pairs {
            trace!("writing key {}", unsafe { str::from_utf8_unchecked(&key) });
            self.output.write(&key).unwrap();
            self.output.write(&val).unwrap();
        }
        self.output.write(b"e").unwrap();
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &'a mut MapSerializer {
    type Ok = Vec<u8>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTuple for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStruct for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
    }
}
