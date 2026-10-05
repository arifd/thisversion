//! Documents and their content evolve independently. One root `into_model()`
//! call migrates both histories and removes their version wrappers.

use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// DOCUMENT HISTORY                                                           //
//============================================================================//

#[derive(VersionFamily)]
pub enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
    V3(DocumentV3),
}

/// Documents originally stored their body as plain text.
pub struct DocumentV1 {
    pub title: String,
    pub body: String,
}

/// Content now has its own version history, independent of document metadata.
pub struct DocumentV2 {
    pub title: String,
    pub content: ContentVersions,
}

impl From<DocumentV1> for DocumentV2 {
    fn from(previous: DocumentV1) -> Self {
        Self {
            title: previous.title,
            // This historical migration targets Content V1 explicitly.
            // Adding newer content versions does not change what it produces.
            content: ContentVersions::V1(ContentV1 {
                text: previous.body,
            }),
        }
    }
}

/// Archiving changes the document schema without changing its content history.
#[derive(VersionBoundary)]
#[thisversion(model = Document)]
pub struct DocumentV3 {
    pub title: String,
    #[thisversion(nested)]
    pub content: ContentVersions,
    pub archived: bool,
}

impl From<DocumentV2> for DocumentV3 {
    fn from(previous: DocumentV2) -> Self {
        Self {
            title: previous.title,
            content: previous.content,
            archived: false,
        }
    }
}

//============================================================================//
// CONTENT HISTORY                                                            //
//============================================================================//

#[derive(VersionFamily)]
pub enum ContentVersions {
    V1(ContentV1),
    V2(ContentV2),
}

/// The first content schema preserves the original plain text representation.
pub struct ContentV1 {
    pub text: String,
}

/// Content can now represent individual paragraphs.
#[derive(VersionBoundary)]
#[thisversion(model = Content)]
pub struct ContentV2 {
    pub paragraphs: Vec<String>,
}

impl From<ContentV1> for ContentV2 {
    fn from(previous: ContentV1) -> Self {
        // Preserve the text exactly, including any embedded line breaks.
        Self {
            paragraphs: vec![previous.text],
        }
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The application sees migrated content with no version wrappers.
#[derive(Debug, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub content: Content,
    pub archived: bool,
}

/// Application-defined content, independent of its persisted schema history.
#[derive(Debug, PartialEq, Eq)]
pub struct Content {
    pub paragraphs: Vec<String>,
}

fn main() {
    let old = DocumentVersions::V1(DocumentV1 {
        title: "First document".to_owned(),
        body: "Written before structured content.".to_owned(),
    });

    // Check one root conversion migrates both document and content histories:
    assert_eq!(
        old.into_model(),
        Document {
            title: "First document".to_owned(),
            content: Content {
                paragraphs: vec!["Written before structured content.".to_owned()]
            },
            archived: false,
        }
    );

    // Even the latest document schema may contain an older content version.
    let current = DocumentVersions::V3(DocumentV3 {
        title: "Archived document".to_owned(),
        content: ContentVersions::V1(ContentV1 {
            text: "Still plain text.".to_owned(),
        }),
        archived: true,
    });
    let model = current.into_model();

    // Check nested migration runs even when no outer migration is needed:
    assert_eq!(
        model,
        Document {
            title: "Archived document".to_owned(),
            content: Content {
                paragraphs: vec!["Still plain text.".to_owned()]
            },
            archived: true,
        }
    );

    let persisted: DocumentVersions = model.into_schema();

    // Check outgoing persistence selects the latest version in both families:
    assert!(matches!(
        persisted,
        DocumentVersions::V3(DocumentV3 {
            content: ContentVersions::V2(_),
            ..
        })
    ));
}
