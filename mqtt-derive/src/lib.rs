//! Macros internal to the `mqtt` library
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use utils::PropertyFieldMeta;

mod decode;
mod default;
mod encode;
mod utils;

/// Generates implementations of `Default`, `Decodeable` and `Into<Vec<u8>>` for a struct with 
/// `#[derive(MqttProperties)]` attribute.
/// 
/// This will only work for structs representing MQTT packet properties, and will only work if:
/// - the properties consist only of fields that are `Option` of one of the following rust datatypes: `u16`, 
/// `u32`, `bool`, `String` or `Vec<u8>`, or a `HashMap<String, String>`
/// - the properties are located within the mqtt::packet module
/// 
/// TODO better error handling, especially using spans to locate issues with individual fields
/// 
#[proc_macro_derive(MqttProperties)]
pub fn mqtt_properties_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;

    // stole this from Jon Gjengset's proc macro workshop:
    // https://github.com/jonhoo/proc-macro-workshop/blob/master/builder/src/lib.rs
    // we're only interested in named fields for now
    let fields = if let syn::Data::Struct(
        syn::DataStruct {fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),..}
    ) = ast.data
    {
        named
    } else {
        panic!("No named fields found in struct {:?}!", name)
    };

    let fields_mapped: Vec<PropertyFieldMeta> = fields.iter()
    .map(|f| {PropertyFieldMeta::from(f)})
    .collect();

    let default_impl = default::generate_default(name, fields);
    let into_impl = encode::generate_encode(name, &fields_mapped);
    let decode_impl = decode::generate_decode(name, &fields_mapped);

    quote! {
        #default_impl

        #into_impl

        #decode_impl
    }.into()
}