#![cfg(feature = "alloc")]

use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// MANUAL INFALLIBLE CHILD CONVERSIONS                                        //
//============================================================================//

// An infallible parent only requires `IntoModel` from a nested child.
// `TryIntoModel` remains an optional adapter rather than a required boundary.
struct ManualChildModel {
    value: u8,
}

struct ManualChildSchema {
    value: u8,
}

impl IntoModel for ManualChildSchema {
    type Model = ManualChildModel;

    fn into_model(self) -> Self::Model {
        ManualChildModel { value: self.value }
    }
}

impl IntoSchema<ManualChildSchema> for ManualChildModel {
    fn into_schema(self) -> ManualChildSchema {
        ManualChildSchema { value: self.value }
    }
}

struct ManualParentModel {
    child: ManualChildModel,
}

#[derive(VersionBoundary)]
#[thisversion(model = ManualParentModel)]
struct ManualParentSchema {
    #[thisversion(nested)]
    child: ManualChildSchema,
}

#[derive(VersionFamily)]
enum ManualParentVersions {
    V1(ManualParentSchema),
}

#[test]
fn infallible_parent_supports_child_without_try_into_model() {
    let model = ManualParentModel {
        child: ManualChildModel { value: 7 },
    };

    let stored: ManualParentVersions = model.into_schema();
    let recovered = stored.into_model();

    assert_eq!(recovered.child.value, 7);
}

//============================================================================//
// RECURSIVE NESTED CONVERSIONS                                               //
//============================================================================//

#[derive(Debug, PartialEq, Eq)]
struct Item {
    text: String,
}

#[derive(VersionBoundary)]
#[thisversion(model = Item)]
struct ItemV2 {
    text: String,
}

impl From<String> for ItemV2 {
    fn from(text: String) -> Self {
        Self { text }
    }
}

#[derive(VersionFamily)]
enum ItemVersions {
    V1(String),
    V2(ItemV2),
}

// A model may participate in more than one version family. Nested schema
// fields therefore select their intended family explicitly.
#[derive(VersionFamily)]
enum AlternateItems {
    Current(ItemV2),
}

#[derive(Debug, PartialEq, Eq)]
struct Group {
    items: Vec<Option<Item>>,
}

// Use an alias here to verify that nested conversion remains trait-driven.
type StoredItems = Vec<Option<ItemVersions>>;

#[derive(VersionBoundary)]
#[thisversion(model = Group)]
struct GroupV1 {
    #[thisversion(nested)]
    items: StoredItems,
}

#[derive(VersionFamily)]
enum GroupVersions {
    V1(GroupV1),
}

#[derive(Debug, PartialEq, Eq)]
struct Root {
    title: String,
    group: Group,
    optional: Option<Vec<Item>>,
}

#[derive(VersionBoundary)]
#[thisversion(model = Root)]
struct RootV1 {
    title: String,

    #[thisversion(nested)]
    group: GroupVersions,

    #[thisversion(nested)]
    optional: Option<Vec<ItemVersions>>,
}

#[derive(VersionFamily)]
enum RootVersions {
    V1(RootV1),
}

#[test]
fn root_conversion_migrates_descendants_and_preserves_containers() {
    let stored = RootVersions::V1(RootV1 {
        title: "root".to_owned(),
        group: GroupVersions::V1(GroupV1 {
            items: vec![
                Some(ItemVersions::V1("old".to_owned())),
                None,
                Some(ItemVersions::V2(ItemV2 {
                    text: "current".to_owned(),
                })),
            ],
        }),
        optional: Some(vec![ItemVersions::V1("optional".to_owned())]),
    });

    let model = stored.into_model();

    // Loading recursively migrates descendants while preserving collection
    // order, absent values, and owned data.
    assert_eq!(
        model,
        Root {
            title: "root".to_owned(),
            group: Group {
                items: vec![
                    Some(Item {
                        text: "old".to_owned(),
                    }),
                    None,
                    Some(Item {
                        text: "current".to_owned(),
                    }),
                ],
            },
            optional: Some(vec![Item {
                text: "optional".to_owned(),
            }]),
        }
    );

    let persisted: RootVersions = model.into_schema();
    let RootVersions::V1(root) = persisted;
    let GroupVersions::V1(group) = root.group;

    // Persisting recursively selects the latest variant of each explicitly
    // chosen version family.
    assert!(matches!(
        group.items.as_slice(),
        [Some(ItemVersions::V2(_)), None, Some(ItemVersions::V2(_))]
    ));
    assert!(matches!(
        root.optional.as_deref(),
        Some([ItemVersions::V2(_)])
    ));
}

#[test]
fn model_can_convert_into_multiple_version_families() {
    let stored: AlternateItems = Item {
        text: "alternative".to_owned(),
    }
    .into_schema();

    assert_eq!(
        stored.into_model(),
        Item {
            text: "alternative".to_owned(),
        }
    );
}

#[test]
fn empty_and_absent_nested_containers_round_trip() {
    let model = Root {
        title: String::new(),
        group: Group { items: vec![] },
        optional: None,
    };

    let stored: RootVersions = model.into_schema();

    assert_eq!(
        stored.into_model(),
        Root {
            title: String::new(),
            group: Group { items: vec![] },
            optional: None,
        }
    );
}

//============================================================================//
// DIRECT CONTAINER CONVERSIONS                                               //
//============================================================================//

#[test]
fn containers_convert_directly_without_a_containing_derive() {
    let stored: StoredItems = vec![Some(ItemVersions::V1("manual".to_owned())), None];

    let models = stored.into_model();

    // Container implementations are available directly to callers and do not
    // depend on traversal generated by a containing derive.
    assert_eq!(
        models,
        vec![
            Some(Item {
                text: "manual".to_owned(),
            }),
            None,
        ]
    );

    let stored: StoredItems = models.into_schema();

    // Direct persistence also reconstructs the latest family variants.
    assert!(matches!(
        stored.as_slice(),
        [Some(ItemVersions::V2(_)), None]
    ));
}
