use quote::quote;

/// Generates a `Default` impl for the derived struct.
pub fn generate_default(
    name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> quote::__private::TokenStream {
    // input for the default() function
    let default_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let val = default_value(f);
        quote! { #name: #val}
    });

    quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#default_fields,)*
                }
            }
        }
    }
}

fn default_value(field: &syn::Field) -> quote::__private::TokenStream {
    //let ty = &field.ty;
    if let syn::Type::Path(ref p) = &field.ty {
        if let Some(segment) = p.path.segments.first() {
            let ty = &segment.ident;
            if ty == "Option" {
                return quote! { None };
            } else if ty == "HashMap" {
                return quote! { std::collections::HashMap::new() };
            } else if ty == "bool" {
                return quote! { true };
            }
        }
    }

    // TODO if we get here, an unsupported data type was encountered and we should
    // report it as an error with the appropriate Span
    unimplemented!()
}
