use super::{graphql_scalar_type_to_rust_type, ident, type_name, AddToOutput, Output, TypeType};
use graphql_parser::schema::Definition::*;
use graphql_parser::schema::TypeDefinition::*;
use graphql_parser::schema::*;
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_query_trails(doc: &Document, out: &mut Output) {
    gen_query_trail(out);

    for def in &doc.definitions {
        match def {
            TypeDefinition(type_def) => match type_def {
                Object(obj) => gen_field_walk_methods(obj, out),
                _ => {}
            },
            _ => {}
        }
    }
}

fn gen_query_trail(out: &mut Output) {
    (quote! {
        /// A wrapper around a `juniper::LookAheadSelection` with methods for each possible child.
        ///
        /// Generated by `juniper-from-schema`.
        pub struct QueryTrail<'a, T, K> {
            look_ahead: Option<&'a juniper::LookAheadSelection<'a>>,
            node_type: std::marker::PhantomData<T>,
            walked: K,
        }

        impl<'a, T> QueryTrail<'a, T, NotWalked> {
            /// Check if the trail is present in the query being executed
            ///
            /// Generated by `juniper-from-schema`.
            pub fn walk(self) -> Option<QueryTrail<'a, T, Walked>> {
                match self.look_ahead {
                    Some(inner) => {
                        Some(QueryTrail {
                            look_ahead: Some(inner),
                            node_type: self.node_type,
                            walked: Walked,
                        })
                    },
                    None => None,
                }
            }
        }

        /// A type used to parameterize `QueryTrail` to know that `walk` has been called.
        pub struct Walked;

        /// A type used to parameterize `QueryTrail` to know that `walk` has *not* been called.
        pub struct NotWalked;

        trait MakeQueryTrail<'a> {
            fn make_query_trail<T>(&'a self) -> QueryTrail<'a, T, NotWalked>;
        }

        impl<'a> MakeQueryTrail<'a> for juniper::LookAheadSelection<'a> {
            fn make_query_trail<T>(&'a self) -> QueryTrail<'a, T, NotWalked> {
                QueryTrail {
                    look_ahead: Some(self),
                    node_type: std::marker::PhantomData,
                    walked: NotWalked,
                }
            }
        }
    })
    .add_to(out);
}

fn gen_field_walk_methods(obj: &ObjectType, out: &mut Output) {
    let name = ident(&obj.name);
    let methods = obj
        .fields
        .iter()
        .map(|field| gen_field_walk_method(field, &out));

    (quote! {
        impl<'a, K> QueryTrail<'a, #name, K> {
            #(#methods)*
        }
    })
    .add_to(out)
}

fn gen_field_walk_method(field: &Field, out: &Output) -> TokenStream {
    let field_type = type_name(&field.field_type);
    let (_, ty) = graphql_scalar_type_to_rust_type(field_type.clone(), &out);
    let field_type = ident(field_type.clone());

    match ty {
        TypeType::Scalar => {
            quote! {}
        }
        TypeType::Type => {
            let name = ident(&field.name.to_snake_case());
            let string_name = &field.name;

            quote! {
                /// Walk the trail into a field.
                ///
                /// Generated by `juniper-from-schema`.
                pub fn #name(&self) -> QueryTrail<'a, #field_type, NotWalked> {
                    use juniper::LookAheadMethods;

                    let child = self.look_ahead.and_then(|la| la.select_child(#string_name));

                    QueryTrail {
                        look_ahead: child,
                        node_type: std::marker::PhantomData,
                        walked: NotWalked,
                    }
                }
            }
        }
    }
}