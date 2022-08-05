use quote::quote;

use crate::utils::{PropertyFieldMeta};

/// Generates an `impl Into<std::vec::Vec<u8>> for #ype_name` where `#struct_name` is the name of the struct with the
/// derive annotation.
pub fn generate_decode(
    name: &syn::Ident,
    fields: &Vec<PropertyFieldMeta>,
) -> quote::__private::TokenStream {

    let decode_fields = fields.iter().map(|f| {
        let prop_path = f.prop_ident_as_path();
        let drep = f.data_rep_as_path();
        let assignment = assignment(f);
        quote!{
            #prop_path => {
                if let #drep(v) = prop.value {
                    #assignment
                }
                Ok(())
            }
        }
    });

    let namestr = name.to_string();

    quote! {
        impl crate::packet::Decodeable for #name {
            fn decode(src: &[u8]) -> std::result::Result<super::DecodingResult<Self>, crate::error::MqttError> {
                
                let mut result = Self::default();
                let bytes_read = super::properties::parse_properties(src, |prop| {
                    match prop.identifier {
                        #(#decode_fields,)*
                        _=> return Err(crate::error::MqttError::Message(
                            format!("Unknown property identifier: [{:?}] for {}", prop.identifier, #namestr)))
                    }
                })?;

                let value = match bytes_read {
                    0 | 1 => None,
                    _=> Some(result)
                };

                Ok(super::DecodingResult{ bytes_read, value })
            }
        }
    }
}

/// Returns the `mqtt::packet::properties::DataRepresentation` variant as an `Ident` for this field along with a
/// `TokenStream` of the value.
/// This is only pseudo-exhausting at the moment!
fn assignment(field: &PropertyFieldMeta) -> quote::__private::TokenStream {
    let fname = &field.name;
    match field.ty_readable.as_str() {
        // don't need this, single bytes are only ever used as bools: "u8" => quote!{ property.value.try_into()? },
        "u16" => quote!{ result.#fname = Some(v) },
        "u32" => quote!{ result.#fname = Some(v) },
        "bool" => quote!{ result.#fname = Some(prop.value.try_into()?) },
        "String" => quote!{ result.#fname = v.value },
        "Vec" => quote!{ result.#fname = Some(v.clone_inner()) },
        "HashMap" => quote!{ 
            result.#fname.insert(v.key.value.unwrap(), v.value.value.unwrap_or(String::new()));
         },
        "QoS" => quote!{ result.#fname = Some(QoS::try_from(v)?) },
        "VariableByteInteger" => quote!{ result.#fname = Some(v) },
        els => panic!("Cannot create decoding for {:?} of type {:?}", field.name, els)
    }
}