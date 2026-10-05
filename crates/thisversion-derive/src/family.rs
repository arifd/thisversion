use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Generics, Ident, Type};

/// A nonempty family of versions in declaration order, oldest to latest.
pub(crate) struct VersionFamily {
    ident: Ident,
    generics: Generics,
    versions: Vec<Version>,
    fallible: bool,
}

/// A version variant and its single schema payload.
struct Version {
    ident: Ident,
    payload: Type,
}

pub(crate) fn parse(input: DeriveInput) -> syn::Result<VersionFamily> {
    let fallible = parse_mode(&input)?;
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let Data::Enum(data) = data else {
        return Err(syn::Error::new_spanned(
            ident,
            "VersionFamily can only be derived for enums",
        ));
    };

    let versions = data
        .variants
        .into_iter()
        .map(|variant| {
            let Fields::Unnamed(fields) = variant.fields else {
                return Err(syn::Error::new_spanned(
                    variant,
                    "version variants must be tuple variants",
                ));
            };

            if fields.unnamed.len() != 1 {
                return Err(syn::Error::new_spanned(
                    fields,
                    "version variants must contain exactly one field",
                ));
            }

            let field = fields.unnamed.into_iter().next().expect("one field");
            Ok(Version {
                ident: variant.ident,
                payload: field.ty,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    if versions.is_empty() {
        return Err(syn::Error::new_spanned(
            ident,
            "VersionFamily requires at least one version",
        ));
    }

    Ok(VersionFamily {
        ident,
        generics,
        versions,
        fallible,
    })
}

/// Generates a fallible model conversion that migrates one family edge at a
/// time, then delegates to the latest schema's model boundary.
pub(crate) fn expand(
    VersionFamily {
        ident,
        generics,
        versions,
        fallible,
    }: VersionFamily,
    runtime: &syn::Path,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let latest = versions.last().expect("validated nonempty family");
    let latest_variant = &latest.ident;
    let latest_type = &latest.payload;

    let from_versions = versions.iter().map(|version| {
        let variant = &version.ident;
        let payload = &version.payload;
        quote! {
            impl #impl_generics ::core::convert::From<#payload>
                for #ident #ty_generics
                #where_clause
            {
                fn from(value: #payload) -> Self {
                    Self::#variant(value)
                }
            }
        }
    });

    let migration_arms = versions.windows(2).map(|pair| {
        let previous_variant = &pair[0].ident;
        let next_variant = &pair[1].ident;
        if fallible {
            quote! {
                Self::#previous_variant(previous) => {
                    let next = Self::#next_variant(::core::convert::TryInto::try_into(previous)?);
                    #runtime::traits::TryIntoModel::try_into_model(next)
                }
            }
        } else {
            quote! {
                Self::#previous_variant(previous) => {
                    let next = Self::#next_variant(::core::convert::Into::into(previous));
                    #runtime::traits::IntoModel::into_model(next)
                }
            }
        }
    });

    let model_conversion = if fallible {
        quote! {
            impl #impl_generics #runtime::traits::TryIntoModel
                for #ident #ty_generics
                #where_clause
            {
                type Model = <#latest_type as #runtime::traits::TryIntoModel>::Model;
                type Error = <#latest_type as #runtime::traits::TryIntoModel>::Error;

                fn try_into_model(self) -> ::core::result::Result<Self::Model, Self::Error> {
                    match self {
                        #(#migration_arms,)*
                        Self::#latest_variant(latest) =>
                            #runtime::traits::TryIntoModel::try_into_model(latest),
                    }
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics #runtime::traits::IntoModel
                for #ident #ty_generics
                #where_clause
            {
                type Model = <#latest_type as #runtime::traits::IntoModel>::Model;

                fn into_model(self) -> Self::Model {
                    match self {
                        #(#migration_arms,)*
                        Self::#latest_variant(latest) =>
                            #runtime::traits::IntoModel::into_model(latest),
                    }
                }
            }

            impl #impl_generics #runtime::traits::TryIntoModel
                for #ident #ty_generics
                #where_clause
            {
                type Model = <#latest_type as #runtime::traits::IntoModel>::Model;
                type Error = ::core::convert::Infallible;

                fn try_into_model(self) -> ::core::result::Result<Self::Model, Self::Error> {
                    ::core::result::Result::Ok(
                        #runtime::traits::IntoModel::into_model(self)
                    )
                }
            }
        }
    };

    quote! {
        #(#from_versions)*
        #model_conversion
    }
}

/// Selects whether family migrations and model conversion can fail.
fn parse_mode(input: &DeriveInput) -> syn::Result<bool> {
    let mut fallible = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("thisversion") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("fallible") {
                return Err(meta.error("expected `fallible` on a version family"));
            }
            if fallible {
                return Err(meta.error("duplicate `fallible`"));
            }
            fallible = true;
            Ok(())
        })?;
    }

    Ok(fallible)
}
