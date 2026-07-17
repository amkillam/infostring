use crate::error::{Error, Result};
use serde::Deserialize;
use serde::de::{self, IntoDeserializer, MapAccess, SeqAccess, Visitor};

#[cfg(not(feature = "std"))]
use crate::alloc::string::ToString;

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_infostr(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub struct Deserializer<'de> {
    parts: core::str::Split<'de, char>,
    current_value: Option<&'de str>,
}

impl<'de> Deserializer<'de> {
    pub fn from_infostr(input: &'de str) -> Self {
        let mut parts = input.split('\\');
        if input.starts_with('\\') {
            parts.next();
        }
        Deserializer {
            parts,
            current_value: None,
        }
    }

    fn pop_value(&mut self) -> Result<&'de str> {
        self.current_value.take().ok_or(Error::Eof)
    }
}

macro_rules! parse_primitive {
    ($method:ident, $type:ty, $visitor_method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let val = self.pop_value()?;
            let parsed = val
                .parse::<$type>()
                .map_err(|_| Error::ParseError(val.to_string()))?;
            visitor.$visitor_method(parsed)
        }
    };
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(InfoStringMapAccess { de: self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(InfoStringSeqAccess { de: self })
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = self.pop_value()?;
        let parsed = match val {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => return Err(Error::ParseError(val.to_string())),
        };
        visitor.visit_bool(parsed)
    }

    parse_primitive!(deserialize_i8, i8, visit_i8);
    parse_primitive!(deserialize_i16, i16, visit_i16);
    parse_primitive!(deserialize_i32, i32, visit_i32);
    parse_primitive!(deserialize_i64, i64, visit_i64);
    parse_primitive!(deserialize_u8, u8, visit_u8);
    parse_primitive!(deserialize_u16, u16, visit_u16);
    parse_primitive!(deserialize_u32, u32, visit_u32);
    parse_primitive!(deserialize_u64, u64, visit_u64);
    parse_primitive!(deserialize_f32, f32, visit_f32);
    parse_primitive!(deserialize_f64, f64, visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = self.pop_value()?;
        let mut chars = val.chars();
        let c = chars.next().ok_or(Error::Eof)?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.pop_value()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.pop_value()?;
        visitor.visit_unit()
    }

    // Forward the rest to any
    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct enum
    }
}

struct InfoStringMapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapAccess<'de> for InfoStringMapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.parts.next() {
            Some(key) if !key.is_empty() => seed.deserialize(key.into_deserializer()).map(Some),
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self.de.parts.next().unwrap_or("");
        self.de.current_value = Some(value);
        let result = seed.deserialize(&mut *self.de);
        self.de.current_value = None;
        result
    }
}

struct InfoStringSeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> SeqAccess<'de> for InfoStringSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        // Clone iterator to peek if we've run out of items without advancing state
        let mut peek = self.de.parts.clone();
        if peek.next().is_none() && self.de.current_value.is_none() {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}
