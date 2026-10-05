use syn::{Data, DeriveInput, Field, Fields, Generics, Ident, Index, Member, Path, Type, Variant};

/// A latest schema's structural boundary with its application model.
///
/// Parsing normalizes Rust's struct/enum syntax into a small representation
/// consumed by expansion. Fields marked `#[thisversion(nested)]` cross the
/// model boundary recursively; ordinary fields move unchanged.
pub(crate) struct VersionBoundary {
    pub(crate) latest: Ident,
    pub(crate) model: Path,
    pub(crate) generics: Generics,
    pub(crate) data: VersionBoundaryData,
    pub(crate) error: Option<Path>,
}

/// The structural forms supported by model conversion.
///
/// Unions are excluded because there is no general safe fieldwise conversion
/// for them.
pub(crate) enum VersionBoundaryData {
    Struct(LatestFields),
    Enum(Vec<LatestVariant>),
}

/// A normalized enum variant.
pub(crate) struct LatestVariant {
    pub(crate) ident: Ident,
    pub(crate) fields: LatestFields,
}

/// A normalized field shape shared by structs and enum variants.
pub(crate) enum LatestFields {
    Named(Vec<LatestField>),
    Unnamed(Vec<LatestField>),
    Unit,
}

/// A field crossing the latest-schema/model boundary.
///
/// `member` is either a named member (`field`) or tuple index (`0`), allowing
/// the same conversion machinery to handle named and unnamed fields.
pub(crate) struct LatestField {
    pub(crate) member: Member,
    pub(crate) ty: Type,
    pub(crate) nested: bool,
    pub(crate) family: Option<Path>,
}

impl VersionBoundaryData {
    /// Iterates over every field in the type.
    pub(crate) fn fields(&self) -> impl Iterator<Item = &LatestField> {
        match self {
            Self::Struct(fields) => EitherFields::Struct(fields.iter()),
            Self::Enum(variants) => {
                EitherFields::Enum(variants.iter().flat_map(|variant| variant.fields.iter()))
            }
        }
    }
}

impl LatestFields {
    /// Iterates over the fields regardless of their syntactic shape.
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, LatestField> {
        match self {
            Self::Named(fields) | Self::Unnamed(fields) => fields.iter(),
            Self::Unit => [].iter(),
        }
    }
}

/// Internal iterator used to avoid allocating while walking `LatestData`.
enum EitherFields<A, B> {
    Struct(A),
    Enum(B),
}

impl<'a, A, B> Iterator for EitherFields<A, B>
where
    A: Iterator<Item = &'a LatestField>,
    B: Iterator<Item = &'a LatestField>,
{
    type Item = &'a LatestField;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Struct(iter) => iter.next(),
            Self::Enum(iter) => iter.next(),
        }
    }
}

pub(crate) fn parse(input: DeriveInput) -> syn::Result<VersionBoundary> {
    let (model, error) = parse_options(&input)?;

    let model = model.ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "model conversion requires `#[thisversion(model = Model)]`",
        )
    })?;

    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let data = match data {
        Data::Struct(data) => VersionBoundaryData::Struct(parse_fields(data.fields)?),

        Data::Enum(data) => VersionBoundaryData::Enum(
            data.variants
                .into_iter()
                .map(parse_variant)
                .collect::<syn::Result<_>>()?,
        ),

        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "model boundaries do not support unions",
            ));
        }
    };

    Ok(VersionBoundary {
        latest: ident,
        model,
        generics,
        data,
        error,
    })
}

/// Reads the model destination and optional final error type.
fn parse_options(input: &DeriveInput) -> syn::Result<(Option<Path>, Option<Path>)> {
    let mut model = None;
    let mut error = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("thisversion") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("model") {
                if model.is_some() {
                    return Err(meta.error("duplicate `model`"));
                }

                model = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("error") {
                if error.is_some() {
                    return Err(meta.error("duplicate `error`"));
                }

                error = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported `thisversion` attribute"))
            }
        })?;
    }

    Ok((model, error))
}

/// Parses one enum variant.
fn parse_variant(variant: Variant) -> syn::Result<LatestVariant> {
    Ok(LatestVariant {
        ident: variant.ident,
        fields: parse_fields(variant.fields)?,
    })
}

/// Normalizes named, unnamed, and unit fields.
fn parse_fields(fields: Fields) -> syn::Result<LatestFields> {
    match fields {
        Fields::Named(fields) => {
            let fields = fields
                .named
                .into_iter()
                .map(|field| {
                    let Field {
                        ident, ty, attrs, ..
                    } = field;

                    parse_field(attrs, ty, Member::Named(ident.expect("named field")))
                })
                .collect::<syn::Result<_>>()?;

            Ok(LatestFields::Named(fields))
        }

        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| {
                    let Field { ty, attrs, .. } = field;

                    parse_field(attrs, ty, Member::Unnamed(Index::from(index)))
                })
                .collect::<syn::Result<_>>()?;

            Ok(LatestFields::Unnamed(fields))
        }

        Fields::Unit => Ok(LatestFields::Unit),
    }
}

/// Parses the versioning semantics attached to one field.
fn parse_field(attrs: Vec<syn::Attribute>, ty: Type, member: Member) -> syn::Result<LatestField> {
    let mut nested = false;
    let mut family = None;

    for attr in &attrs {
        if !attr.path().is_ident("thisversion") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("nested") {
                return Err(meta.error("expected `nested` on a field"));
            }

            if nested {
                return Err(meta.error("duplicate `nested`"));
            }

            nested = true;

            if meta.input.peek(syn::Token![=]) {
                family = Some(meta.value()?.parse()?);
            }

            Ok(())
        })?;
    }

    Ok(LatestField {
        member,
        ty,
        nested,
        family,
    })
}
