#![allow(unused)]
#![allow(dead_code)]
mod auto;
mod enum_darling;
mod enum_from;

use crate::auto::{process_auto_debug, process_auto_deref};
use crate::enum_darling::process_enum_from_darling;
use crate::enum_from::process_enum_from;
use proc_macro::TokenStream;
use syn::DeriveInput;

//DeriveInput {
//     attrs: [],
//     vis: Visibility::Inherited,
//     ident: Ident {
//         ident: "Direction",
//         span: #0 bytes(160..169),
//     },
//     generics: Generics {
//         lt_token: None,
//         params: [],
//         gt_token: None,
//         where_clause: None,
//     },
//     data: Data::Enum {
//         enum_token: Enum,
//         brace_token: Brace,
//         variants: [
//             Variant {
//                 attrs: [],
//                 ident: Ident {
//                     ident: "UP",
//                     span: #0 bytes(176..178),
//                 },
//                 fields: Fields::Unnamed {
//                     paren_token: Paren,
//                     unnamed: [
//                         Field {
//                             attrs: [],
//                             vis: Visibility::Inherited,
//                             mutability: FieldMutability::None,
//                             ident: None,
//                             colon_token: None,
//                             ty: Type::Path {
//                                 qself: None,
//                                 path: Path {
//                                     leading_colon: None,
//                                     segments: [
//                                         PathSegment {
//                                             ident: Ident {
//                                                 ident: "DirectionUp",
//                                                 span: #0 bytes(179..190),
//                                             },
//                                             arguments: PathArguments::None,
//                                         },
//                                     ],
//                                 },
//                             },
//                         },
//                     ],
//                 },
//                 discriminant: None,
//             },
//             Comma,
//             Variant {
//                 attrs: [],
//                 ident: Ident {
//                     ident: "Down",
//                     span: #0 bytes(197..201),
//                 },
//                 fields: Fields::Unit,
//                 discriminant: None,
//             },
//             Comma,
//         ],
//     },
// }
#[proc_macro_derive(EnumFrom)]
pub fn derive_enum_from(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    // println!("{:#?}", input);
    process_enum_from(input).into()
}

#[proc_macro_derive(EnumFromDarling)]
pub fn derive_enum_from_darling(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    println!("{input:#?}");
    process_enum_from_darling(input).into()
}

#[proc_macro_derive(AutoDeref, attributes(deref))]
pub fn derive_auto_deref(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    println!("{input:#?}");
    process_auto_deref(input).into()
}

#[proc_macro_derive(AutoDebug, attributes(debug))]
pub fn derive_auto_debug(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    println!("{input:#?}");
    process_auto_debug(input).into()
}
