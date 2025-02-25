use serde::ser::{self, Serialize};
use std::fmt::Display;
use std::io::Write;
use std::{f32, f64, io};

use crate::error::{Error, Result};

/// Attempts to serialize the input as a JSON5 string (actually a JSON string).
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer::new(Vec::<u8>::new());
    value.serialize(&mut serializer)?;
    let output = String::from_utf8(serializer.take_output())?;
    Ok(output)
}

/// Attempts to serialize the input as JSON5 string into the I/O stream.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

struct Serializer<W> {
    output: InnerWriter<W>,
    // TODO settings for formatting (single vs double quotes, whitespace etc)
}

impl<W> Serializer<W> {
    fn new(writer: W) -> Self {
        Self {
            output: InnerWriter {
                writer,
                last_byte: 0,
            },
        }
    }

    fn take_output(self) -> W {
        self.output.writer
    }
}

struct InnerWriter<W> {
    writer: W,
    last_byte: u8,
}

impl<W> InnerWriter<W> {
    fn ends_with(&self, c: char) -> bool {
        self.last_byte == (c as u8)
    }
}

impl<W: io::Write> io::Write for InnerWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;
        if written > 0 {
            self.last_byte = buf[written - 1];
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: io::Write> Serializer<W> {
    fn write_display<T>(&mut self, v: &T) -> Result<()>
    where
        T: Display,
    {
        write!(&mut self.output, "{}", v)?;
        Ok(())
    }
}

impl<W: io::Write> ser::Serializer for &mut Serializer<W> {
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
        self.write_display(&v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_display(&v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        if v == f32::INFINITY {
            self.output.write_all(b"Infinity")?;
        } else if v == f32::NEG_INFINITY {
            self.output.write_all(b"-Infinity")?;
        } else if v.is_nan() {
            self.output.write_all(b"NaN")?;
        } else {
            self.write_display(&v)?;
        }
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v == f64::INFINITY {
            self.output.write_all(b"Infinity")?;
        } else if v == f64::NEG_INFINITY {
            self.output.write_all(b"-Infinity")?;
        } else if v.is_nan() {
            self.output.write_all(b"NaN")?;
        } else {
            self.write_display(&v)?;
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        write!(&mut self.output, "\"{}\"", escape(v))?;
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        unimplemented!() // TODO
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output.write_all(b"null")?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
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
        self.output.write_all(b"{")?;
        variant.serialize(&mut *self)?; // TODO drop the quotes where possible
        self.output.write_all(b":")?;
        value.serialize(&mut *self)?;
        self.output.write_all(b"}")?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output.write_all(b"[")?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output.write_all(b"{")?;
        variant.serialize(&mut *self)?;
        self.output.write_all(b":[")?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output.write_all(b"{")?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output.write_all(b"{")?;
        variant.serialize(&mut *self)?;
        self.output.write_all(b":{")?;
        Ok(self)
    }
}

impl<W: io::Write> ser::SerializeSeq for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output.write_all(b",")?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_all(b"]")?;
        Ok(())
    }
}

impl<W: io::Write> ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<W: io::Write> ser::SerializeTupleStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<W: io::Write> ser::SerializeTupleVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        self.output.write_all(b"]}")?;
        Ok(())
    }
}

impl<W: io::Write> ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output.write_all(b",")?;
        }
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.write_all(b":")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_all(b"}")?;
        Ok(())
    }
}

impl<W: io::Write> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<W: io::Write> ser::SerializeStructVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        self.output.write_all(b"}}")?;
        Ok(())
    }
}

fn escape(v: &str) -> String {
    v.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', c],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            '\\' => vec!['\\', '\\'],
            '\u{0008}' => vec!['\\', 'b'],
            '\u{000c}' => vec!['\\', 'f'],
            c => vec![c],
        })
        .collect()
}
