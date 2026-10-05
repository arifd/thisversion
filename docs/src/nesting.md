# Choosing how nested schemas evolve

Imagine a document editor that originally stored a title and body as plain
text. Later, content becomes a separate part of the application: it contains
paragraphs and can be marked as reviewed.

Document metadata and content can now evolve independently. There are two ways
to represent that relationship in a persisted schema:

- **Store the child family**  
  Use this when the child should carry its own version and evolve independently
  of the parent.

- **Pin a child schema**  
  Use this when the parent's stored format intentionally contains one particular
  representation of the child.

The difference determines whether the parent automatically follows new child
versions or remains tied to a specific child format.

## Give the child its own history

Start by giving content an independent version history. Content V1 stores
paragraphs; V2 adds review state.

A version family names those stored forms, while `VersionBoundary` connects the
newest schema to the application's `Content` model. Migrating old content makes
it unreviewed:

```rust
# extern crate thisversion;
use thisversion::{VersionBoundary, VersionFamily};
use thisversion::traits::{IntoModel, IntoSchema};

#[derive(VersionFamily)]
enum ContentVersions {
    V1(ContentV1),
    V2(ContentV2),
}

struct ContentV1 {
    paragraphs: Vec<String>,
}

#[derive(VersionBoundary)]
#[thisversion(model = Content)]
struct ContentV2 {
    paragraphs: Vec<String>,
    reviewed: bool,
}

struct Content {
    paragraphs: Vec<String>,
    reviewed: bool,
}

impl From<ContentV1> for ContentV2 {
    fn from(old: ContentV1) -> Self {
        Self {
            paragraphs: old.paragraphs,
            reviewed: false,
        }
    }
}
```

Content now has a history that can advance independently of any document that
contains it.

## Store the child family in the parent

The original document schema has a plain `body`. Its next version replaces that
field with `ContentVersions`:

```rust
# extern crate thisversion;
# #[derive(VersionFamily)]
# enum ContentVersions {
#     V1(ContentV1),
#     V2(ContentV2),
# }
#
# struct ContentV1 {
#     paragraphs: Vec<String>,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Content)]
# struct ContentV2 {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# struct Content {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# impl From<ContentV1> for ContentV2 {
#     fn from(old: ContentV1) -> Self {
#         Self {
#             paragraphs: old.paragraphs,
#             reviewed: false,
#         }
#     }
# }
use thisversion::{VersionBoundary, VersionFamily};
use thisversion::traits::{IntoModel, IntoSchema};

#[derive(VersionFamily)]
enum DocumentVersions {
    V0(DocumentV0),
    V1(DocumentV1),
}

struct DocumentV0 {
    title: String,
    body: String,
}

#[derive(VersionBoundary)]
#[thisversion(model = Document)]
struct DocumentV1 {
    title: String,
    #[thisversion(nested)]
    content: ContentVersions,
}

struct Document {
    title: String,
    content: Content,
}

impl From<DocumentV0> for DocumentV1 {
    fn from(old: DocumentV0) -> Self {
        Self {
            title: old.title,
            content: ContentVersions::V1(ContentV1 {
                paragraphs: vec![old.body],
            }),
        }
    }
}
```

The migration explicitly creates `ContentVersions::V1`. Naming the variant is
important: the old document body corresponds to Content V1, and that historical
fact should not change when Content V2 or V3 is added later.

The `nested` marker tells [`VersionBoundary`][VersionBoundary] to convert the field
between `ContentVersions` and the application's `Content` model. Converting the
outer family therefore migrates both histories:

```rust
# extern crate thisversion;
# #[derive(VersionFamily)]
# enum ContentVersions {
#     V1(ContentV1),
#     V2(ContentV2),
# }
#
# struct ContentV1 {
#     paragraphs: Vec<String>,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Content)]
# struct ContentV2 {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# impl From<ContentV1> for ContentV2 {
#     fn from(old: ContentV1) -> Self {
#         Self {
#             paragraphs: old.paragraphs,
#             reviewed: false,
#         }
#     }
# }
#
# struct Content {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# #[derive(VersionFamily)]
# enum DocumentVersions {
#     V0(DocumentV0),
#     V1(DocumentV1),
# }
#
# struct DocumentV0 {
#     title: String,
#     body: String,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Document)]
# struct DocumentV1 {
#     title: String,
#     #[thisversion(nested)]
#     content: ContentVersions,
# }
#
# struct Document {
#     title: String,
#     content: Content,
# }
#
# impl From<DocumentV0> for DocumentV1 {
#     fn from(old: DocumentV0) -> Self {
#         Self {
#             title: old.title,
#             content: ContentVersions::V1(ContentV1 {
#                 paragraphs: vec![old.body],
#             }),
#         }
#     }
# }
use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

let old = DocumentVersions::V0(DocumentV0 {
    title: "Draft".into(),
    body: "First paragraph".into(),
});

let document = old.into_model();

assert_eq!(document.content.paragraphs, ["First paragraph"]);
assert!(!document.content.reviewed);

let stored: DocumentVersions = document.into_schema();

assert!(matches!(
    stored,
    DocumentVersions::V1(DocumentV1 {
        content: ContentVersions::V2(_),
        ..
    })
));
```

Document V1 may also contain Content V1 directly. Loading it still migrates the
child to the current `Content` model, even though the document itself is already
at its latest version.

Saving takes the opposite path and writes the latest variant of each family.
If content later gains V3, Document V1 can remain the latest document schema;
new writes will simply contain Content V3.

That independence also affects compatibility. A reader must understand the new
child version before it can read documents that contain it, even if the parent
version itself has not changed.

## Pin the parent to a child schema

Sometimes the parent should not follow the child's latest stored
representation. Its persisted format may deliberately contain the fields of a
particular child schema without storing a child version tag.

In that design, store the child schema directly and name its family in the
`nested` attribute:

```rust
# extern crate thisversion;
# #[derive(VersionFamily)]
# enum ContentVersions {
#     V1(ContentV1),
#     V2(ContentV2),
# }
#
# struct ContentV1 {
#     paragraphs: Vec<String>,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Content)]
# struct ContentV2 {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# struct Content {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# impl From<ContentV1> for ContentV2 {
#     fn from(old: ContentV1) -> Self {
#         Self {
#             paragraphs: old.paragraphs,
#             reviewed: false,
#         }
#     }
# }
use thisversion::{VersionBoundary, VersionFamily};

#[derive(VersionBoundary)]
#[thisversion(model = PinnedDocument)]
struct PinnedDocumentSchema {
    title: String,
    #[thisversion(nested = ContentVersions)]
    content: ContentV1,
}

struct PinnedDocument {
    title: String,
    content: Content,
}

// NOTE: this is a down migration. V2 -> v1
impl From<ContentV2> for ContentV1 {
    fn from(
        ContentV2 {
            paragraphs,
            reviewed: _, // This is lossy!
        }: ContentV2,
    ) -> Self {
        Self { paragraphs }
    }
}
```

Here, `nested = ContentVersions` tells the derive that `ContentV1` belongs to
the family that knows how to migrate it into the current `Content` model.

Loading still follows the child history, so Content V1 becomes a current
`Content` with `reviewed = false`.

Saving is different. The parent remains pinned to Content V1, so the current
Content V2 representation must be converted back to V1. In this example, that
conversion cannot preserve `reviewed`:

```rust
# extern crate thisversion;
# struct ContentV1 {
#     paragraphs: Vec<String>,
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = Content)]
# struct ContentV2 {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# struct Content {
#     paragraphs: Vec<String>,
#     reviewed: bool,
# }
#
# impl From<ContentV1> for ContentV2 {
#     fn from(old: ContentV1) -> Self {
#         Self {
#             paragraphs: old.paragraphs,
#             reviewed: false,
#         }
#     }
# }
#
# #[derive(VersionFamily)]
# enum ContentVersions {
#     V1(ContentV1),
#     V2(ContentV2),
# }
#
# impl From<ContentV2> for ContentV1 {
#     fn from(latest: ContentV2) -> Self {
#         Self {
#             paragraphs: latest.paragraphs,
#         }
#     }
# }
#
# #[derive(VersionBoundary)]
# #[thisversion(model = PinnedDocument)]
# struct PinnedDocumentSchema {
#     title: String,
#     #[thisversion(nested = ContentVersions)]
#     content: ContentV1,
# }
#
# struct PinnedDocument {
#     title: String,
#     content: Content,
# }
use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

let document = PinnedDocument {
    title: "Reviewed draft".into(),
    content: Content {
        paragraphs: vec!["First paragraph".into()],
        reviewed: true,
    },
};

let stored: PinnedDocumentSchema = document.into_schema();
let reloaded = stored.into_model();

assert!(!reloaded.content.reviewed);
```

The round trip deliberately loses review state because the pinned format has
nowhere to store it.

If that information must survive, evolve the parent to store Content V2 or the
child family instead. [`IntoSchema`][IntoSchema] is infallible. Here,
`From<ContentV2> for ContentV1` lets the model save through the pinned parent,
deliberately losing review state. Without that reverse conversion, the derive
still supports loading, but saving to `PinnedDocumentSchema` does not compile.
If an export to the pinned format must reject reviewed content, provide a
separate fallible conversion that returns `Result<PinnedDocumentSchema, Error>`.

## Choose between the two designs

The choice comes down to what the parent's persisted contract should mean:

- **Store `ContentVersions`**  
  When content owns its version history and the parent should accept new child
  versions without introducing a new parent version.

- **Store `ContentV1` with `nested = ContentVersions`**  
  When the parent intentionally promises that particular child representation,
  even as the child family continues to evolve elsewhere.

For either design, test old child versions nested inside the latest parent. For
a pinned child, also test the full model → schema → model round trip so any
information lost by the older representation is explicit.

[VersionBoundary]: https://docs.rs/thisversion/latest/thisversion/derive.VersionBoundary.html
[IntoSchema]: https://docs.rs/thisversion/latest/thisversion/traits/trait.IntoSchema.html
