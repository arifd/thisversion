use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Generics, Ident, Member, Path};

use super::parse::{LatestField, LatestFields, VersionBoundary, VersionBoundaryData};

/// Generates conversions in both directions between the latest schema and its
/// application model.
pub(crate) fn expand(boundary: VersionBoundary, runtime: &Path) -> TokenStream {
    let into_model = expand_into_model(&boundary, runtime);
    let into_schema = expand_into_schema(boundary, runtime);

    quote! {
        #into_model
        #into_schema
    }
}

/// Generates the latest-schema -> model conversion.
fn expand_into_model(boundary: &VersionBoundary, runtime: &Path) -> TokenStream {
    let VersionBoundary {
        latest,
        model,
        generics,
        data,
        error,
    } = boundary;

    let body = map_data(
        quote!(#latest),
        model_in_constructor(model),
        data,
        |field, value| to_model(field, value, error.is_some(), runtime),
    );

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Some(error) = error {
        quote! {
            impl #impl_generics #runtime::traits::TryIntoModel for #latest #ty_generics #where_clause {
                type Model = #model;
                type Error = #error;

                fn try_into_model(self) -> ::core::result::Result<Self::Model, Self::Error> {
                    ::core::result::Result::Ok(#body)
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics #runtime::traits::IntoModel for #latest #ty_generics #where_clause {
                type Model = #model;

                fn into_model(self) -> Self::Model {
                    #body
                }
            }

            impl #impl_generics #runtime::traits::TryIntoModel for #latest #ty_generics #where_clause {
                type Model = #model;
                type Error = ::core::convert::Infallible;

                fn try_into_model(self) -> ::core::result::Result<Self::Model, Self::Error> {
                    ::core::result::Result::Ok(
                        #runtime::traits::IntoModel::into_model(self)
                    )
                }
            }
        }
    }
}

/// Generates model -> schema conversion through the latest schema value.
fn expand_into_schema(boundary: VersionBoundary, runtime: &Path) -> TokenStream {
    let VersionBoundary {
        latest,
        model,
        generics,
        data,
        error,
    } = boundary;

    let schema = fresh_generic_ident(&generics);
    let has_nested_fields = data.fields().any(|field| field.nested);

    let body = map_data(
        model_in_pattern(&model),
        quote!(#latest),
        &data,
        |field, value| {
            if !field.nested {
                return value;
            }

            let ty = &field.ty;
            let model_ty = nested_model_type(field, error.is_some(), runtime);

            quote! {
                <#schema as __ThisversionSaveNested<#model_ty, #ty>>::save(#value)
            }
        },
    );

    let generics = schema_generics(&latest, generics, &data, error.is_some(), runtime, &schema);
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let into_schema = quote! {
        impl #impl_generics #runtime::traits::IntoSchema<#schema> for #model #where_clause {
            fn into_schema(self) -> #schema {
                #schema::from(#body)
            }
        }
    };

    if !has_nested_fields {
        return into_schema;
    }

    quote! {
        const _: () = {
            trait __ThisversionSaveNested<__Model, __FieldSchema> {
                fn save(value: __Model) -> __FieldSchema;
            }

            impl<__Schema, __Model, __FieldSchema> __ThisversionSaveNested<__Model, __FieldSchema> for __Schema
            where
                __Model: #runtime::traits::IntoSchema<__FieldSchema>,
            {
                fn save(value: __Model) -> __FieldSchema {
                    #runtime::traits::IntoSchema::into_schema(value)
                }
            }

            #into_schema
        };
    }
}

/// Strips generic arguments from the model path for pattern matching.
fn model_in_pattern(model: &Path) -> TokenStream {
    let leading_colon = &model.leading_colon;
    let last = model.segments.last().expect("model path is nonempty");
    let ident = &last.ident;
    let prefix = model.segments.iter().take(model.segments.len() - 1);

    quote!(#leading_colon #(#prefix::)* #ident)
}

/// Formats a model path into a turbofish constructor expression
/// (e.g. `Container::<Item>`).
fn model_in_constructor(model: &Path) -> TokenStream {
    let leading_colon = &model.leading_colon;
    let last = model.segments.last().expect("model path is nonempty");
    let ident = &last.ident;
    let prefix = model.segments.iter().take(model.segments.len() - 1);

    match &last.arguments {
        syn::PathArguments::AngleBracketed(arguments) => {
            quote!(#leading_colon #(#prefix::)* #ident :: #arguments)
        }
        _ => quote!(#leading_colon #(#prefix::)* #ident),
    }
}

/// Builds the generics required by the model -> schema conversion.
fn schema_generics(
    latest: &Ident,
    mut generics: Generics,
    data: &VersionBoundaryData,
    fallible: bool,
    runtime: &Path,
    schema: &Ident,
) -> Generics {
    let (_, ty_generics, _) = generics.split_for_impl();
    let ty_generics = quote!(#ty_generics);

    generics.params.push(syn::parse_quote!(#schema));
    let where_clause = generics.make_where_clause();

    where_clause.predicates.push(syn::parse_quote! {
        #schema: ::core::convert::From<#latest #ty_generics>
    });

    for field in data.fields().filter(|field| field.nested) {
        let field_ty = &field.ty;
        let model_ty = nested_model_type(field, fallible, runtime);

        where_clause.predicates.push(syn::parse_quote! {
            #schema: __ThisversionSaveNested<#model_ty, #field_ty>
        });
    }

    generics
}

/// Chooses a type parameter name that does not shadow a schema generic.
fn fresh_generic_ident(generics: &Generics) -> Ident {
    let mut suffix = 0;
    loop {
        let name = if suffix == 0 {
            "__ThisversionSchema".to_string()
        } else {
            format!("__ThisversionSchema{suffix}")
        };
        let candidate = format_ident!("{name}");
        let conflicts = generics.params.iter().any(|param| match param {
            syn::GenericParam::Lifetime(p) => p.lifetime.ident == candidate,
            syn::GenericParam::Type(p) => p.ident == candidate,
            syn::GenericParam::Const(p) => p.ident == candidate,
        });

        if !conflicts {
            return candidate;
        }
        suffix += 1;
    }
}

/// The application type reached by loading a nested field.
fn nested_model_type(field: &LatestField, fallible: bool, runtime: &Path) -> TokenStream {
    let model_trait = if fallible {
        quote!(TryIntoModel)
    } else {
        quote!(IntoModel)
    };

    if let Some(family) = &field.family {
        quote!(<#family as #runtime::traits::#model_trait>::Model)
    } else {
        let ty = &field.ty;
        quote!(<#ty as #runtime::traits::#model_trait>::Model)
    }
}

/// Maps one struct or enum representation to another.
fn map_data(
    source: TokenStream,
    target: TokenStream,
    data: &VersionBoundaryData,
    mut map_field: impl FnMut(&LatestField, TokenStream) -> TokenStream,
) -> TokenStream {
    match data {
        VersionBoundaryData::Struct(fields) => {
            let arm = map_fields(source, target, fields, &mut map_field);
            quote!(match self { #arm })
        }
        VersionBoundaryData::Enum(variants) => {
            let arms = variants.iter().map(|variant| {
                let ident = &variant.ident;
                map_fields(
                    quote!(#source::#ident),
                    quote!(#target::#ident),
                    &variant.fields,
                    &mut map_field,
                )
            });

            quote!(match self { #(#arms,)* })
        }
    }
}

/// Maps a single struct or enum-variant shape from `source` to `target`.
fn map_fields(
    source: TokenStream,
    target: TokenStream,
    fields: &LatestFields,
    map_field: &mut impl FnMut(&LatestField, TokenStream) -> TokenStream,
) -> TokenStream {
    match fields {
        LatestFields::Named(fields) => {
            let bindings = fields.iter().map(|field| {
                let Member::Named(name) = &field.member else {
                    unreachable!("named fields contain named members");
                };
                let binding = binding_ident(&field.member);
                quote!(#name: #binding)
            });

            let values = fields.iter().map(|field| {
                let Member::Named(name) = &field.member else {
                    unreachable!("named fields contain named members");
                };
                let binding = binding_ident(&field.member);
                let value = map_field(field, quote!(#binding));
                quote!(#name: #value)
            });

            quote! {
                #source { #(#bindings,)* } => #target { #(#values,)* }
            }
        }
        LatestFields::Unnamed(fields) => {
            let bindings: Vec<_> = fields
                .iter()
                .map(|field| binding_ident(&field.member))
                .collect();

            let values = fields
                .iter()
                .zip(&bindings)
                .map(|(field, binding)| map_field(field, quote!(#binding)));

            quote! {
                #source( #(#bindings,)* ) => #target( #(#values,)* )
            }
        }
        LatestFields::Unit => quote!(#source => #target),
    }
}

/// Converts a latest-schema field into its model representation.
fn to_model(
    field: &LatestField,
    value: TokenStream,
    fallible: bool,
    runtime: &Path,
) -> TokenStream {
    if !field.nested {
        return value;
    }

    let value = if let Some(family) = &field.family {
        let ty = &field.ty;
        quote!(<#family as ::core::convert::From<#ty>>::from(#value))
    } else {
        value
    };

    if fallible {
        quote!(#runtime::traits::TryIntoModel::try_into_model(#value)?)
    } else {
        quote!(#runtime::traits::IntoModel::into_model(#value))
    }
}

/// Produces a private binding for a generated field pattern.
fn binding_ident(member: &Member) -> Ident {
    match member {
        Member::Named(ident) => {
            format_ident!("__thisversion_{ident}", span = Span::mixed_site())
        }
        Member::Unnamed(index) => {
            format_ident!("__thisversion_{}", index.index, span = Span::mixed_site())
        }
    }
}
