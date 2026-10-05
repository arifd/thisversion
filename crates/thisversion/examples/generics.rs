//! A document library manages titles and archiving while applications choose
//! their own content representation. Document migrations move that content
//! unchanged, without requiring it to implement `Clone` or `Copy`.

use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// VERSIONS                                                                   //
//============================================================================//

/// The document's schema history, shared by every content representation.
///
/// These versions describe changes to the surrounding document fields. The
/// content representation stays the same throughout this migration history.
#[derive(VersionFamily)]
pub enum DocumentVersions<Content> {
    V1(DocumentV1<Content>),
    V2(DocumentV2<Content>),
    V3(DocumentV3<Content>),
}

//============================================================================//
// V1                                                                         //
//============================================================================//

/// The original document schema stores a title and application-defined body.
pub struct DocumentV1<Content> {
    pub title: String,
    pub body: Content,
}

//============================================================================//
// V2                                                                         //
//============================================================================//

/// The second schema adds archiving independently of the body representation.
pub struct DocumentV2<Content> {
    pub title: String,
    pub body: Content,
    pub archived: bool,
}

impl<Content> From<DocumentV1<Content>> for DocumentV2<Content> {
    fn from(previous: DocumentV1<Content>) -> Self {
        Self {
            title: previous.title,
            body: previous.body,
            // Documents written before archiving existed remain active.
            archived: false,
        }
    }
}

//============================================================================//
// V3                                                                         //
//============================================================================//

/// The latest schema renames `body` to `content` and matches the application
/// model, allowing the derive to move fields in both directions.
#[derive(VersionBoundary)]
#[thisversion(model = Document<Content>)]
pub struct DocumentV3<Content> {
    pub title: String,
    pub content: Content,
    pub archived: bool,
}

impl<Content> From<DocumentV2<Content>> for DocumentV3<Content> {
    fn from(previous: DocumentV2<Content>) -> Self {
        Self {
            title: previous.title,
            content: previous.body,
            archived: previous.archived,
        }
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The application keeps its concrete content type after migration, so it can
/// use content-specific operations without interpreting a generic payload.
#[derive(Debug, PartialEq, Eq)]
pub struct Document<Content> {
    pub title: String,
    pub content: Content,
    pub archived: bool,
}

/// An editor's content representation: an ordered sequence of styled text.
///
/// This application-defined type deliberately does not implement `Clone` or
/// `Copy`; document migrations only need ownership of its value.
#[derive(Debug, PartialEq, Eq)]
struct RichText {
    spans: Vec<TextSpan>,
}

/// A run of text whose characters share the same formatting.
#[derive(Debug, PartialEq, Eq)]
struct TextSpan {
    text: String,
    bold: bool,
}

fn main() {
    // A Markdown application stores its content as an ordinary string.
    let markdown = DocumentVersions::V1(DocumentV1 {
        title: "Getting started".to_owned(),
        body: "Read the **installation guide**.".to_owned(),
    });
    let markdown: Document<String> = markdown.into_model();

    // Check migration preserves Markdown and defaults old documents to active:
    assert_eq!(
        markdown,
        Document {
            title: "Getting started".to_owned(),
            content: "Read the **installation guide**.".to_owned(),
            archived: false,
        }
    );

    // An editor uses the same history with structured, owned content.
    let rich_text = DocumentVersions::V2(DocumentV2 {
        title: "Release announcement".to_owned(),
        body: RichText {
            spans: vec![
                TextSpan {
                    text: "Introducing ".to_owned(),
                    bold: false,
                },
                TextSpan {
                    text: "our new editor".to_owned(),
                    bold: true,
                },
            ],
        },
        archived: true,
    });
    let rich_text: Document<RichText> = rich_text.into_model();

    // Check migration preserves the editor's formatting and archive state:
    assert_eq!(
        rich_text,
        Document {
            title: "Release announcement".to_owned(),
            content: RichText {
                spans: vec![
                    TextSpan {
                        text: "Introducing ".to_owned(),
                        bold: false,
                    },
                    TextSpan {
                        text: "our new editor".to_owned(),
                        bold: true,
                    },
                ],
            },
            archived: true,
        }
    );

    // Each application can return its model to the latest schema while
    // retaining its concrete content representation.
    let markdown: DocumentVersions<String> = markdown.into_schema();
    let rich_text: DocumentVersions<RichText> = rich_text.into_schema();

    // Check both content representations enter the family at V3:
    assert!(matches!(markdown, DocumentVersions::V3(_)));
    assert!(matches!(rich_text, DocumentVersions::V3(_)));
}
