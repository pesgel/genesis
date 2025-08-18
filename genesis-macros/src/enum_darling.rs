use darling::ast::{Data, Fields, Style};
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
struct EnumFromDarling {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumVariants, ()>,
}

#[derive(Debug, FromVariant)]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<EnumVariantsFields>,
}

#[derive(Debug, FromField)]
struct EnumVariantsFields {
    ty: syn::Type,
}

pub(crate) fn process_enum_from_darling(input: syn::DeriveInput) -> TokenStream {
    let EnumFromDarling {
        ident,
        generics,
        data: Data::Enum(data),
    } = EnumFromDarling::from_derive_input(&input).expect("failed to process enum")
    else {
        panic!("enums are not supported");
    };
    let from_impls = data.iter().map(|variant| {
        let var = &variant.ident;
        let style = &variant.fields.style;

        match style {
            Style::Tuple if variant.fields.len() == 1 => {
                let field = variant.fields.iter().next().expect("empty tuple struct");
                let ty = &field.ty;
                quote! {
                    impl #generics From<#ty> for #ident #generics {
                         fn from(item: #ty) -> Self {
                            #ident::#var(item)
                        }
                    }
                }
            }
            _ => quote! {},
        }
    });

    quote! {
        #(#from_impls)*
    }
}
