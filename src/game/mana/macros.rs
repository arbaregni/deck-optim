#[macro_export]
macro_rules! impl_serde_traits {

    { $struct_name:ident { serialize => $ser_meth:path, deserialize => $deser_meth:path } } => {

        struct Visitor;
        impl <'de> serde::de::Visitor<'de> for Visitor {
            type Value = $struct_name;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a mana cost")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                let mp = $ser_meth(&value)
                    .map_err(|e| E::custom(e))?;

                Ok(mp)
            }
        }
        impl <'de> serde::Deserialize<'de> for $struct_name {
            fn deserialize<D>(deser: D) -> Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                deser.deserialize_str(Visitor)
            }
        }
        impl serde::Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                let s = $deser_meth(self);
                serializer.serialize_str(&s)
            }
        }

    };
}
