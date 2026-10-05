use thisversion::traits::{IntoSchema, TryIntoModel};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// CONTENT                                                                    //
//============================================================================//

/// Content migration rejects text that the new representation cannot accept.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentError;

#[derive(VersionFamily)]
#[thisversion(fallible)]
pub enum ContentVersions {
    V1(ContentV1),
    V2(ContentV2),
}

/// Early content was stored as a single string.
pub struct ContentV1 {
    pub text: String,
}

/// The current schema groups text into paragraphs.
#[derive(VersionBoundary)]
#[thisversion(model = Content, error = ContentError)]
pub struct ContentV2 {
    pub paragraphs: Vec<String>,
}

impl TryFrom<ContentV1> for ContentV2 {
    type Error = ContentError;

    fn try_from(previous: ContentV1) -> Result<Self, Self::Error> {
        // Legacy content could contain NUL characters. The new editor cannot
        // represent them, and silently removing them would lose information.
        if previous.text.contains('\0') {
            return Err(ContentError);
        }
        Ok(Self {
            paragraphs: vec![previous.text],
        })
    }
}

//============================================================================//
// DOCUMENT                                                                   //
//============================================================================//

/// Historical document migration requires a nonempty title.
#[derive(Debug, PartialEq, Eq)]
pub struct TitleError;

/// The caller handles one error type regardless of which conversion failed.
#[derive(Debug, PartialEq, Eq)]
pub enum DocumentError {
    Title(TitleError),
    Content(ContentError),
}

impl From<TitleError> for DocumentError {
    fn from(error: TitleError) -> Self {
        Self::Title(error)
    }
}

impl From<ContentError> for DocumentError {
    fn from(error: ContentError) -> Self {
        Self::Content(error)
    }
}

/// The latest snapshot chooses `DocumentError` to cover title migration and
/// nested content conversion.
#[derive(VersionFamily)]
#[thisversion(fallible)]
pub enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
}

pub struct DocumentV1 {
    pub title: String,
    pub content: ContentV1,
}

/// Even after the outer migration, the nested content can still be historical.
#[derive(VersionBoundary)]
#[thisversion(model = Document, error = DocumentError)]
pub struct DocumentV2 {
    pub title: String,
    #[thisversion(nested)]
    pub content: ContentVersions,
    pub archived: bool,
}

impl TryFrom<DocumentV1> for DocumentV2 {
    type Error = TitleError;

    fn try_from(previous: DocumentV1) -> Result<Self, Self::Error> {
        if previous.title.trim().is_empty() {
            return Err(TitleError);
        }
        Ok(Self {
            title: previous.title,
            content: previous.content.into(),
            archived: false,
        })
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The loaded document contains only application models.
#[derive(Debug, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub content: Content,
    pub archived: bool,
}

/// Paragraphs are ready for the application after successful migration.
#[derive(Debug, PartialEq, Eq)]
pub struct Content {
    pub paragraphs: Vec<String>,
}

fn main() -> Result<(), DocumentError> {
    let stored = DocumentVersions::V1(DocumentV1 {
        title: "First document".to_owned(),
        content: ContentV1 {
            text: "Original text.".to_owned(),
        },
    });

    // Both historical migration and recursive conversion use DocumentError.
    let model = stored.try_into_model()?;

    // Check both schema histories reach their application models:
    assert_eq!(model.content.paragraphs, ["Original text."]);

    let stored = DocumentVersions::V2(DocumentV2 {
        title: "Already current document".to_owned(),
        content: ContentVersions::V1(ContentV1 {
            text: "Still stored as an old content snapshot.".to_owned(),
        }),
        archived: true,
    });
    let model = stored.try_into_model()?;

    // Check the child history migrates even when the outer schema is latest:
    assert_eq!(
        model.content.paragraphs,
        ["Still stored as an old content snapshot."]
    );

    // Saving selects the latest version from both nested families:
    let persisted: DocumentVersions = model.into_schema();
    assert!(matches!(
        persisted,
        DocumentVersions::V2(DocumentV2 {
            content: ContentVersions::V2(_),
            ..
        })
    ));

    Ok(())
}
