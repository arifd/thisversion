//============================================================================//
// PINNED CONTENT WITHOUT A REVERSE CONVERSION                                //
//============================================================================//

mod load_only {
    use thisversion::traits::IntoModel;
    use thisversion::{VersionBoundary, VersionFamily};

    struct ContentV1 {
        value: u32,
    }

    #[derive(VersionBoundary)]
    #[thisversion(model = Content)]
    struct ContentV2 {
        value: u32,
        reviewed: bool,
    }

    impl From<ContentV1> for ContentV2 {
        fn from(old: ContentV1) -> Self {
            Self {
                value: old.value,
                reviewed: false,
            }
        }
    }

    #[derive(VersionFamily)]
    enum ContentVersions {
        V1(ContentV1),
        V2(ContentV2),
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Content {
        value: u32,
        reviewed: bool,
    }

    // Document V1 pins Content V1, even though Content V2 is now latest.
    // There is deliberately no From<ContentV2> for ContentV1.
    #[derive(VersionBoundary)]
    #[thisversion(model = Document)]
    struct DocumentV1 {
        #[thisversion(nested = ContentVersions)]
        content: ContentV1,
    }

    #[derive(VersionFamily)]
    enum DocumentVersions {
        V1(DocumentV1),
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Document {
        content: Content,
    }

    #[test]
    fn document_loads_pinned_content_without_a_reverse_conversion() {
        let stored = DocumentVersions::V1(DocumentV1 {
            content: ContentV1 { value: 7 },
        });

        assert_eq!(
            stored.into_model(),
            Document {
                content: Content {
                    value: 7,
                    reviewed: false,
                },
            }
        );
    }
}

//============================================================================//
// TWO PINNED DOCUMENT VERSIONS                                               //
//============================================================================//

mod load_and_save {
    use thisversion::traits::{IntoModel, IntoSchema};
    use thisversion::{VersionBoundary, VersionFamily};

    struct ContentV1 {
        value: u32,
    }

    struct ContentV2 {
        value: u32,
        reviewed: bool,
    }

    #[derive(VersionBoundary)]
    #[thisversion(model = Content)]
    struct ContentV3 {
        value: u32,
        reviewed: bool,
        highlighted: bool,
    }

    impl From<ContentV1> for ContentV2 {
        fn from(old: ContentV1) -> Self {
            Self {
                value: old.value,
                reviewed: false,
            }
        }
    }

    impl From<ContentV2> for ContentV3 {
        fn from(old: ContentV2) -> Self {
            Self {
                value: old.value,
                reviewed: old.reviewed,
                highlighted: false,
            }
        }
    }

    impl From<ContentV3> for ContentV2 {
        fn from(latest: ContentV3) -> Self {
            Self {
                value: latest.value,
                reviewed: latest.reviewed,
            }
        }
    }

    impl From<ContentV2> for ContentV1 {
        fn from(previous: ContentV2) -> Self {
            Self {
                value: previous.value,
            }
        }
    }

    #[derive(VersionFamily)]
    enum ContentVersions {
        V1(ContentV1),
        V2(ContentV2),
        V3(ContentV3),
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Content {
        value: u32,
        reviewed: bool,
        highlighted: bool,
    }

    // The first document format pins the first content format.
    struct DocumentV1 {
        content: ContentV1,
    }

    // The next document format pins Content V2, while Content V3 is latest.
    #[derive(VersionBoundary)]
    #[thisversion(model = Document)]
    struct DocumentV2 {
        #[thisversion(nested = ContentVersions)]
        content: ContentV2,
    }

    impl From<DocumentV1> for DocumentV2 {
        fn from(old: DocumentV1) -> Self {
            Self {
                content: ContentV2::from(old.content),
            }
        }
    }

    impl From<DocumentV2> for DocumentV1 {
        fn from(latest: DocumentV2) -> Self {
            Self {
                content: ContentV1::from(latest.content),
            }
        }
    }

    #[derive(VersionFamily)]
    enum DocumentVersions {
        V1(DocumentV1),
        V2(DocumentV2),
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Document {
        content: Content,
    }

    #[test]
    fn old_document_migrates_through_both_content_versions() {
        let stored = DocumentVersions::V1(DocumentV1 {
            content: ContentV1 { value: 11 },
        });

        assert_eq!(
            stored.into_model(),
            Document {
                content: Content {
                    value: 11,
                    reviewed: false,
                    highlighted: false,
                },
            }
        );
    }

    #[test]
    fn saving_to_older_document_runs_two_content_downgrades() {
        let document = Document {
            content: Content {
                value: 11,
                reviewed: true,
                highlighted: true,
            },
        };

        let latest: DocumentV2 = document.into_schema();

        // Check Content V3 -> V2 loses highlighting but preserves review state:
        assert!(latest.content.reviewed);

        let older = DocumentV1::from(latest);

        // Check Content V3 -> V2 -> V1 also loses review state:
        assert_eq!(
            DocumentVersions::V1(older).into_model().content,
            Content {
                value: 11,
                reviewed: false,
                highlighted: false,
            }
        );
    }
}
