//! Demonstrates a parent schema pinned to a child snapshot that used to be
//! latest but has since become historical. Loading migrates the pinned
//! snapshot forward; saving requires an explicit downgrade conversion.

use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// DOCUMENT HISTORY                                                           //
//============================================================================//

#[derive(VersionFamily)]
enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
}

struct DocumentV1 {
    title: String,
    content: ContentV2,
}

/// The parent remains pinned to Content V2 after the child family advances to
/// V3. Naming the family lets loading migrate that snapshot forward.
#[derive(VersionBoundary)]
#[thisversion(model = Document)]
struct DocumentV2 {
    title: String,
    #[thisversion(nested = ContentVersions)]
    content: ContentV2,
}

impl From<DocumentV1> for DocumentV2 {
    fn from(previous: DocumentV1) -> Self {
        Self {
            title: previous.title,
            content: previous.content,
        }
    }
}

//============================================================================//
// CONTENT HISTORY                                                            //
//============================================================================//

#[derive(VersionFamily)]
enum ContentVersions {
    V1(ContentV1),
    V2(ContentV2),
    V3(ContentV3),
}

struct ContentV1 {
    text: String,
}

/// This was once the latest content schema, so the parent persisted it directly
/// instead of storing `ContentVersions`.
struct ContentV2 {
    paragraphs: Vec<String>,
}

/// Content later gained review state, making V2 a historical schema.
#[derive(VersionBoundary)]
#[thisversion(model = Content)]
struct ContentV3 {
    paragraphs: Vec<String>,
    reviewed: bool,
}

impl From<ContentV1> for ContentV2 {
    fn from(previous: ContentV1) -> Self {
        Self {
            paragraphs: vec![previous.text],
        }
    }
}

impl From<ContentV2> for ContentV3 {
    fn from(previous: ContentV2) -> Self {
        Self {
            paragraphs: previous.paragraphs,
            reviewed: false,
        }
    }
}

// This manual downgrade is required because DocumentV2 persists ContentV2.
// It intentionally drops review state when saving through this old schema.
impl From<ContentV3> for ContentV2 {
    fn from(latest: ContentV3) -> Self {
        Self {
            paragraphs: latest.paragraphs,
        }
    }
}

#[derive(Debug)]
struct Content {
    paragraphs: Vec<String>,
    reviewed: bool,
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

#[derive(Debug)]
struct Document {
    title: String,
    content: Content,
}

fn main() {
    let stored = DocumentVersions::V1(DocumentV1 {
        title: "Pinned document".to_owned(),
        content: ContentV2 {
            paragraphs: vec!["Stored as V2".to_owned()],
        },
    });

    let mut model: Document = stored.into_model();
    assert_eq!(model.content.paragraphs, ["Stored as V2"]);
    assert!(!model.content.reviewed);

    // The pinned format cannot persist review state; the downgrade drops it:
    model.content.reviewed = true;
    let persisted: DocumentVersions = model.into_schema();
    assert!(matches!(
        persisted,
        DocumentVersions::V2(DocumentV2 {
            content: ContentV2 { paragraphs },
            ..
        }) if paragraphs == ["Stored as V2"]
    ));
}
