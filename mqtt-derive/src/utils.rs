use quote::format_ident;

pub struct PropertyFieldMeta {
    pub name: syn::Ident,
    pub ty: syn::Type,
    pub ty_readable: String,
    pub optional: bool,
    pub map: bool,
    pub prop_ident: String,
}

impl PropertyFieldMeta {

    pub fn prop_ident_as_path(&self) -> syn::ExprPath {
        build_path(vec![
            "crate", 
            "packet", 
            "properties", 
            "PropertyIdentifier", 
            self.prop_ident.as_str()])
    }

    pub fn data_rep_as_path(&self) -> syn::ExprPath {
        let variant = match self.ty_readable.as_str() {
            "u8" => "Byte",
            "u16" => "TwoByteInt",
            "u32" => match self.name.to_string().as_str() {
                "subscription_identifier" => "VariByteInt",
                _ => "FourByteInt",
            },
            "bool" => "Byte",
            "String" => "UTF8",
            "Vec" => "BinaryData",
            "HashMap" => "UTF8Pair",
            "QoS" => "Byte",
            "VariableByteInteger" => "VariByteInt",
            els => panic!("Cannot convert {:?} of type {:?}", self.name, els)
        };

        build_path(vec![
            "crate",
            "packet", 
            "properties", 
            "DataRepresentation", 
            variant,
        ])
    }
}

impl From<&syn::Field> for PropertyFieldMeta {
    fn from(field: &syn::Field) -> Self {
        map_field_meta(field)
    }
}

fn map_field_meta(field: &syn::Field) -> PropertyFieldMeta {
    let name = match &field.ident {
        Some(id) => id.to_owned(),
        None => format_ident!("unknown"), // FIXME
    };
    let prop_ident = map_enum_variant(&name.to_string());
    let (ty, optional, map) = extract_type(field);

    let ty_readable = match &ty {
        syn::Type::Path(p) => p.path.segments[0].ident.to_string(),
        _ => String::from("unknown"), // FIXME this should be an error or at least lead to ignoring this field alltogether
    };

    PropertyFieldMeta {
        name,
        ty,
        ty_readable,
        optional,
        map,
        prop_ident,
    }
}

fn extract_type(field: &syn::Field) -> (syn::Type, bool, bool) {
    if let syn::Type::Path(ref p) = &field.ty {
        if let Some(segment) = p.path.segments.first() {
            let ty = &segment.ident;
            let is_map = ty.to_string() == "HashMap".to_string();
            if ty == "Option" {
                if let syn::PathArguments::AngleBracketed(ref ab) = segment.arguments {
                    if let syn::GenericArgument::Type(ref t) = ab.args.first().unwrap() {
                        return (t.to_owned(), true, is_map);
                    }
                }
                // FIXME add more sophisticated handling for HashMaps and whatnot
                return (field.ty.to_owned(), true, is_map);
            } else {
                return (field.ty.to_owned(), false, is_map);
            }
        }
    }

    // this isn't right, we should return an error here...
    return (field.ty.to_owned(), false, false);
}

// simply reformats from `abc_def_ghi` to `AbcDefGhi`.
fn map_enum_variant(field_name: &String) -> String {
    let mut result = String::new();
    for part in field_name.split('_') {
        let mut chars = part.chars();
        if let Some(c) = chars.next() {
            result.push(c.to_ascii_uppercase());
        }
        result.push_str(chars.as_str());
    }
    result
}

pub fn build_path(elements: Vec<&str>) -> syn::ExprPath {
    let mut prop_path:syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> = syn::punctuated::Punctuated::new();
    for e in elements {
        prop_path.push(syn::PathSegment{ ident: format_ident!("{}", e), arguments: syn::PathArguments::None });
    }

    syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: prop_path,
        }
    }
}