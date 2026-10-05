use serde::{Deserialize, Serialize};
use thisversion::traits::IntoSchema;
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// HISTORICAL DATA                                                            //
//============================================================================//

const V1_JSON: &str = r#"{
  "version": "V1",
  "data": {
    "title": "First document",
    "body": "Written before archiving existed."
  }
}"#;

const V2_JSON: &str = r#"{
  "version": "V2",
  "data": {
    "title": "Second document",
    "body": "This one has an archived field.",
    "archived": true
  }
}"#;

const V3_JSON: &str = r#"{
  "version": "V3",
  "data": {
    "title": "Second document",
    "content": "This one has an archived field.",
    "archived": true
  }
}"#;

//============================================================================//
// VERSIONS                                                                   //
//============================================================================//

/// Every supported schema version of `Document`.
///
/// The derive sequences the adjacent `From` migrations and wraps latest values.
#[derive(Deserialize, Serialize, VersionFamily)]
#[serde(tag = "version", content = "data")]
pub enum DocumentVersions {
    V1(DocumentV1),
    V2(DocumentV2),
    V3(DocumentV3),
}

//============================================================================//
// V1                                                                         //
//============================================================================//

/// The first version of `Document`.
///
/// Documents initially consisted only of a title and body.
#[derive(Deserialize, Serialize)]
pub struct DocumentV1 {
    pub title: String,
    pub body: String,
}

//============================================================================//
// V2                                                                         //
//============================================================================//

/// The second version adds support for archiving documents.
#[derive(Deserialize, Serialize)]
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

/// The latest version renames `body` to `content`.
///
/// Matching model fields let the derive generate both conversion directions and
/// the infallible `TryIntoModel` used by the Serde adapter.
#[derive(Deserialize, Serialize, VersionBoundary)]
#[thisversion(model = Document)]
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

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

/// The application model we want to use throughout the codebase.
///
/// This example leaves Serde off the application model so callers explicitly
/// choose the schema representation at the persistence boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub content: String,
    pub archived: bool,
}

fn deserialize_document(json: &str) -> Document {
    use thisversion::traits::IntoModel;

    serde_json::from_str::<DocumentVersions>(json)
        .unwrap()
        .into_model()
}

fn serialize_document(document: &Document) -> String {
    use thisversion::serde::serialize_into_schema;

    #[derive(Serialize)]
    struct Record<'a> {
        #[serde(serialize_with = "serialize_into_schema::<DocumentVersions, _>")]
        document: &'a Document,
    }

    serde_json::to_string(&Record { document }).unwrap()
}

fn main() {
    let document = deserialize_document(V1_JSON);

    // Check the field helper preserves the record and version-family wrappers:
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&serialize_document(&document)).unwrap(),
        serde_json::json!({
            "document": {
                "version": "V3",
                "data": {
                    "title": "First document",
                    "content": "Written before archiving existed.",
                    "archived": false,
                },
            },
        })
    );

    // When persisted again, the application value is converted to the latest
    // schema representation and wrapped in its version family.
    let versions: DocumentVersions = document.into_schema();

    // Check outgoing data uses the latest version.
    assert!(matches!(versions, DocumentVersions::V3(_)));

    // Check the latest wire representation preserves the migrated document.
    assert_eq!(
        serde_json::to_value(&versions).unwrap(),
        serde_json::json!({
            "version": "V3",
            "data": {
                "title": "First document",
                "content": "Written before archiving existed.",
                "archived": false,
            },
        })
    );

    // But we can just as easily deserialize any supported version:
    assert_eq!(
        deserialize_document(V2_JSON),
        Document {
            title: "Second document".to_owned(),
            content: "This one has an archived field.".to_owned(),
            archived: true,
        }
    );
    assert_eq!(
        deserialize_document(V3_JSON),
        Document {
            title: "Second document".to_owned(),
            content: "This one has an archived field.".to_owned(),
            archived: true,
        }
    );
}
