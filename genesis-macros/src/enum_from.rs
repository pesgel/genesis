use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn process_enum_from(input: DeriveInput) -> TokenStream {
    // step1. 获取ident
    let ident = &input.ident;
    // 获取范型数据
    let generics = &input.generics;
    // step2. 获取variants
    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => panic!("EnumFrom can only be derived for enums"),
    };
    // 对于每个variant获取ident和fields
    let from_impls = variants.iter().map(|variant| {
        let var = &variant.ident;
        match &variant.fields {
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    quote! {}
                } else {
                    let field = fields
                        .unnamed
                        .first()
                        .expect("Unnamed fields are not supported");
                    let ty = &field.ty;
                    quote! {
                        impl #generics From<#ty> for #ident #generics {
                            fn from(item: #ty) -> Self {
                                #ident::#var(item)
                            }
                        }
                    }
                }
            }
            syn::Fields::Unit => quote! {},
            syn::Fields::Named(_) => quote! {},
        }
    });
    // step3.
    // quote return proc-macro2 TokenStream so we need to convert it to TokenStream
    quote! {
        #(#from_impls)*
    }
}

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
