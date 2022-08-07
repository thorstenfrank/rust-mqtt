use quote::{format_ident, quote};

use crate::utils::PropertyFieldMeta;

/// Generates an `impl From<SRC_TYPE> for std::vec::Vec<u8> where `SRC_TYPE` is the annotated type.
pub fn generate_encode(
    name: &syn::Ident,
    fields: &Vec<PropertyFieldMeta>,
) -> quote::__private::TokenStream {
    let into_fields = fields.iter().map(|f| quote_field(f));

    quote! {
        impl From<#name> for std::vec::Vec<u8> {
            fn from(src: #name) -> Self {
                let mut result: std::vec::Vec<u8> = Vec::new();

                #(#into_fields;)*

                super::encode_and_insert(
                    crate::types::VariableByteInteger::from(result.len() as u32),
                    0,
                    &mut result
                );
                result
            }
        }
    }
}

fn quote_field(field: &PropertyFieldMeta) -> quote::__private::TokenStream {
    let name = &field.name;
    let prop_ident = field.prop_ident_as_path();

    let (drepr, dval) = map_data_types(field);
    
    let assign_and_encode = quote!{
        let val = crate::packet::properties::DataRepresentation::#drepr(#dval);
        super::properties::encode_and_append_property(#prop_ident, val, &mut result);
    };

    if field.map {
        // we only support HashMap<String, String> at the moment
        return quote! {
            for (k, v) in src.#name {
                #assign_and_encode
            }
        };
    }

    match field.optional {
        true => quote!{
            if let Some(v) = src.#name {
                #assign_and_encode
            }
        },
        false => quote!{
            let v = drc.#name;
            #assign_and_encode
        },
    }
}

/// Returns the `mqtt::packet::properties::DataRepresentation` variant as an `Ident` for this field along with a
/// `TokenStream` of the value.
/// This is only pseudo-exhausting at the moment!
fn map_data_types(field: &PropertyFieldMeta) -> (syn::Ident, quote::__private::TokenStream) {
    match field.ty_readable.as_str() {
        "u8" => (format_ident!("{}", "Byte"), quote!{ v }),
        "u16" => (format_ident!("{}", "TwoByteInt"), quote!{ v }),
        "u32" => match field.name.to_string().as_str() {
            // special handling, the only u32 of the propertes that is encoded as a variable byte integer
            "subscription_identifier" => (
                format_ident!("{}", "VariByteInt"),
                quote! { crate::types::VariableByteInteger{ value: v} },
            ),
            _ => (format_ident!("{}", "FourByteInt"), quote!{ v }),
        },
        "bool" => (
            format_ident!("{}", "Byte"), 
            quote!{
                match v {
                    true => 1,
                    false => 0,
                }
            }
        ),
        "String" => (
            format_ident!("{}", "UTF8"),
            quote! { crate::types::UTF8String::from(v) },
        ),
        "Vec" => (
            format_ident!("{}", "BinaryData"),
            quote! { crate::types::BinaryData::new(v).unwrap() },
        ),
        "HashMap" => (
            format_ident!("{}", "UTF8Pair"),
            quote!{ crate::types::UTF8StringPair::new(k, v) }
        ),
        "QoS" => (format_ident!("{}", "Byte"), quote!{ v.into() }),
        "VariableByteInteger" => (format_ident!("{}", "VariByteInt"), quote!{ v }),
        els => panic!("Cannot create encoding for [{:?}] of type {:?}", field.name, els)
    }
}
