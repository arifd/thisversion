use thisversion::VersionBoundary;
use thisversion::traits::{IntoModel, IntoSchema};

#[derive(Debug, PartialEq, Eq)]
struct NamedModel {
    id: u32,
    label: String,
}

#[derive(VersionBoundary)]
#[thisversion(model = NamedModel)]
struct NamedSnapshot {
    id: u32,
    label: String,
}

#[derive(Debug, PartialEq, Eq)]
struct TupleModel(u32, String);

#[derive(VersionBoundary)]
#[thisversion(model = TupleModel)]
struct TupleSnapshot(u32, String);

#[derive(Debug, PartialEq, Eq)]
struct UnitModel;

#[derive(VersionBoundary)]
#[thisversion(model = UnitModel)]
struct UnitSnapshot;

#[derive(Debug, PartialEq, Eq)]
enum EnumModel {
    Unit,
    Tuple(u32, String),
    Struct { first: u16, second: String },
}

#[derive(VersionBoundary)]
#[thisversion(model = EnumModel)]
enum EnumSnapshot {
    Unit,
    Tuple(u32, String),
    Struct { first: u16, second: String },
}

#[test]
fn named_struct_fields_round_trip() {
    let original = NamedModel {
        id: 7,
        label: "named".to_owned(),
    };

    let snapshot: NamedSnapshot = original.into_schema();
    let recovered: NamedModel = snapshot.into_model();

    assert_eq!(
        recovered,
        NamedModel {
            id: 7,
            label: "named".to_owned(),
        }
    );
}

#[test]
fn tuple_struct_fields_round_trip() {
    let original = TupleModel(11, "tuple".to_owned());

    let snapshot: TupleSnapshot = original.into_schema();
    let recovered: TupleModel = snapshot.into_model();

    assert_eq!(recovered, TupleModel(11, "tuple".to_owned()));
}

#[test]
fn unit_struct_round_trip() {
    let snapshot: UnitSnapshot = UnitModel.into_schema();
    let recovered: UnitModel = snapshot.into_model();

    assert_eq!(recovered, UnitModel);
}

#[test]
fn unit_tuple_and_struct_enum_variants_round_trip() {
    let unit: EnumSnapshot = EnumModel::Unit.into_schema();
    let tuple: EnumSnapshot = EnumModel::Tuple(13, "tuple variant".to_owned()).into_schema();
    let structure: EnumSnapshot = EnumModel::Struct {
        first: 17,
        second: "struct variant".to_owned(),
    }
    .into_schema();

    assert_eq!(unit.into_model(), EnumModel::Unit);
    assert_eq!(
        tuple.into_model(),
        EnumModel::Tuple(13, "tuple variant".to_owned())
    );
    assert_eq!(
        structure.into_model(),
        EnumModel::Struct {
            first: 17,
            second: "struct variant".to_owned(),
        }
    );
}
