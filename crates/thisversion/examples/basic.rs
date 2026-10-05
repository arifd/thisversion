use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

/// Every supported schema version of `Document`.
///
/// `VersionFamily` generates the historical migration path through the variants
/// and the conversion into the application model.
#[derive(VersionFamily)]
pub enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
    V3(DocumentV3),
}

//============================================================================//
// V1                                                                         //
//============================================================================//

/// The initial version of a `Document` only required a `title` and a `body`.
pub struct DocumentV1 {
    pub title: String,
    pub body: String,
}

//============================================================================//
// V2                                                                         //
//============================================================================//

pub struct DocumentV2 {
    pub title: String,
    pub body: String,
    pub archived: bool,
}

impl From<DocumentV1> for DocumentV2 {
    fn from(prev: DocumentV1) -> Self {
        Self {
            title: prev.title,
            body: prev.body,
            // V2 added the ability to archive documents.
            // Existing documents are active by default when migrated to V2.
            archived: false,
        }
    }
}

//============================================================================//
// V3                                                                         //
//============================================================================//

// `VersionBoundary` defines the bidirectional boundary between the latest
// schema and the application model.
//
// This completes the migration from any previous version in the family to the
// application model.
#[derive(VersionBoundary)]
#[thisversion(model = Document)]
pub struct DocumentV3 {
    pub title: String,
    pub content: String,
    pub archived: bool,
}

impl From<DocumentV2> for DocumentV3 {
    fn from(prev: DocumentV2) -> Self {
        Self {
            title: prev.title,
            // The latest version renamed `body` to `content`.
            content: prev.body,
            archived: prev.archived,
        }
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The application model, independent of any historical version complexity.
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

    // ...it can be migrated all the way into the application model.
    let model = versions.into_model();

    assert_eq!(model.title, "Hello");
    assert_eq!(model.content, "Hello, world!");
    assert!(!model.archived);

    // The schema derive also lets `Document` convert back into the family.
    let versions = model.into_schema();

    // Application values always enter the family through its latest version.
    assert!(matches!(versions, DocumentVersions::V3(_)));

    // Every supported version converts through the application model.
    let model = versions.into_model();
    assert_eq!(model.content, "Hello, world!");
}
