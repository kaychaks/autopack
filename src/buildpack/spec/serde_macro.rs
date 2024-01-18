/// Macro to create a module to custom (de)serialization of a field of type `Option<String>`
/// where a `None` is serialized using the function `def_fn`
/// which must have the type signature `() -> String`.
///
/// During deserialization `def_fn` will be used to convert a value
/// mathching the return of `def_fn` to a `None`.
///
/// All this to ensure that reflexivity propery
/// of encode and decode holds.
///
/// i.e. `deserialize . serialize = id`
#[macro_export]
macro_rules! ser_deser_str_with_def {
    ($module: ident, $def_fn: ident) => {
        struct $module {}

        impl $module {
            fn serialize<S>(x: &Option<String>, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match x {
                    Some(str) => s.serialize_str(str),
                    None => {
                        let def_str = $def_fn();
                        s.serialize_str(&def_str)
                    }
                }
            }

            fn deserialize<'de, D>(d: D) -> Result<Option<String>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Option::<String>::deserialize(d)?.and_then(|f| {
                    if f == $def_fn() {
                        None
                    } else {
                        Some(f)
                    }
                }))
            }
        }
    };
}
