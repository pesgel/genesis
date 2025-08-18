use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deref))]
struct AutoDerefInfo {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), AutoDerefFieldsInfo>,
    #[darling(default)]
    mutable: bool,
    #[darling(default)]
    field: Option<syn::Ident>,
}

#[derive(Debug, FromField)]
struct AutoDerefFieldsInfo {
    ty: syn::Type,
    ident: Option<syn::Ident>,
}

pub(crate) fn process_auto_deref(derive_input: DeriveInput) -> TokenStream {
    let AutoDerefInfo {
        ident,
        generics,
        data: Data::Struct(fields),
        mutable,
        field,
    } = AutoDerefInfo::from_derive_input(&derive_input).unwrap()
    else {
        panic!("AutoDeref only works on structs")
    };
    let (fd, ty) = if let Some(field) = field {
        match fields.iter().find(|f| f.ident.as_ref().unwrap() == &field) {
            Some(f) => (field, &f.ty),
            None => panic!("field {field:?} not found in the data structure"),
        }
    } else if fields.len() == 1 {
        let f = fields.iter().next().unwrap();
        (f.ident.as_ref().unwrap().clone(), &f.ty)
    } else {
        panic!("AutoDeref only works on structs with 1 field or with field attribute")
    };

    let mut code = vec![quote! {
        impl #generics std::ops::Deref for #ident #generics {
            type Target = #ty;
            fn deref(&self) -> &Self::Target {
                &self.#fd
            }
        }
    }];
    if mutable {
        code.push(quote! {
            impl #generics std::ops::DerefMut for #ident #generics {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#fd
                }
            }
        })
    }
    quote! {
        #(#code)*
    }
}

#[derive(Debug, FromDeriveInput)]
struct AutoDebugInfo {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), AutoDebugFieldsInfo>,
}

#[derive(Debug, FromField)]
#[darling(attributes(debug))]
struct AutoDebugFieldsInfo {
    ident: Option<syn::Ident>,
    #[darling(default)]
    skip: bool,
}

pub(crate) fn process_auto_debug(derive_input: DeriveInput) -> TokenStream {
    let AutoDebugInfo {
        ident,
        generics,
        data: Data::Struct(fields),
    } = AutoDebugInfo::from_derive_input(&derive_input)
        .expect("AutoDerefInfo only works on structs")
    else {
        panic!("AutoDerefInfo only works on structs")
    };

    let fields = fields.iter().map(|field| {
        let ident = &field.ident.as_ref().unwrap();
        let skip = field.skip;
        if skip {
            quote! {}
        } else {
            quote! {
                .field(stringify!(#ident), &self.#ident)
            }
        }
    });
    quote! {
        impl ::core::fmt::Debug for #ident #generics {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
            {
                f.debug_struct(stringify!(#ident))
                        #(#fields)*
                        .finish()
            }
        }
    }
}
