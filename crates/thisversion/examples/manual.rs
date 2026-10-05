use thisversion::traits::{IntoModel, IntoSchema};

//============================================================================//
// VERSIONS                                                                   //
//============================================================================//

/// A document in any supported schema version.
pub enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
    V3(DocumentV3),
}

//============================================================================//
// MIGRATION                                                                  //
//============================================================================//

/// A manual family conversion can make its migration path explicit.
impl IntoModel for DocumentVersions {
    type Model = Document;

    fn into_model(self) -> Self::Model {
        match self {
            Self::V1(v1) => Self::V2(DocumentV2::from(v1)).into_model(),
            Self::V2(v2) => Self::V3(DocumentV3::from(v2)).into_model(),
            Self::V3(v3) => v3.into_model(),
        }
    }
}

//============================================================================//
// V1                                                                         //
//============================================================================//

/// The first version of `Document`.
///
/// Documents initially consisted only of a title and body.
pub struct DocumentV1 {
    pub title: String,
    pub body: String,
}

//============================================================================//
// V2                                                                         //
//============================================================================//

/// The second version adds support for archiving documents.
pub struct DocumentV2 {
    pub title: String,
    pub body: String,
    pub archived: bool,
}

/// Existing documents are active by default when migrated to `V2`.
impl From<DocumentV1> for DocumentV2 {
    fn from(prev: DocumentV1) -> Self {
        Self {
            title: prev.title,
            body: prev.body,
            archived: false,
        }
    }
}

//============================================================================//
// V3                                                                         //
//============================================================================//

/// The latest schema version renames `body` to `content`.
pub struct DocumentV3 {
    pub title: String,
    pub content: String,
    pub archived: bool,
}

/// Migrating from `V2` preserves the document while adopting the new field
/// name.
impl From<DocumentV2> for DocumentV3 {
    fn from(prev: DocumentV2) -> Self {
        Self {
            title: prev.title,
            content: prev.body,
            archived: prev.archived,
        }
    }
}

/// Defines how the latest schema representation of `DocumentVersions` is
/// converted into the application's `Document` model.
///
/// This conversion crosses from the latest schema into the application model.
impl IntoModel for DocumentV3 {
    type Model = Document;

    fn into_model(self) -> Self::Model {
        Document {
            title: self.title,
            content: self.content,
            archived: self.archived,
        }
    }
}

/// Defines how the application's `Document` model is converted into the
/// `DocumentVersions` family.
///
/// This direction is independent of `IntoModel`: applications that only consume
/// a version family do not need to implement it.
///
/// Application values enter the family through its latest schema
/// representation.
impl IntoSchema<DocumentVersions> for Document {
    fn into_schema(self) -> DocumentVersions {
        DocumentVersions::V3(DocumentV3 {
            title: self.title,
            content: self.content,
            archived: self.archived,
        })
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The application model we want to use throughout the codebase.
pub struct Document {
    pub title: String,
    pub content: String,
    pub archived: bool,
}

fn main() {
    // A historical version was decoded from somewhere...
    let v1 = DocumentV1 {
        title: "Hello".to_owned(),
        body: "Hello, world!".to_owned(),
    };

    // Once wrapped in its version family...
    let versions = DocumentVersions::V1(v1);

    // ...it can be migrated directly into the application model.
    //
    // `IntoModel` first migrates the value to `DocumentV3` and then crosses
    // the application boundary using
    // `DocumentV3::into_model`.
    let document = versions.into_model();

    assert_eq!(document.title, "Hello");
    assert_eq!(document.content, "Hello, world!");
    assert!(!document.archived);

    // Because `Document` also implements `IntoSchema<DocumentVersions>`,
    // it can be converted back into the version family.
    //
    // Application values enter the family through its latest schema
    // representation; no historical migration is involved.
    let versions = document.into_schema();

    // Application values are therefore represented by the latest variant
    // when converted into the version family.
    assert!(matches!(versions, DocumentVersions::V3(_)));

    // Every supported version converts through the application model.
    let reloaded = versions.into_model();
    assert_eq!(reloaded.content, "Hello, world!");
}
