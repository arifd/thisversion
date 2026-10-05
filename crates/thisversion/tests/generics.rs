use thisversion::traits::{IntoModel, IntoSchema};
use thisversion::{VersionBoundary, VersionFamily};

//============================================================================//
// GENERIC MODEL BOUNDARIES                                                   //
//============================================================================//

struct Name(String);

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

struct Application<'a, K, V, const N: usize> {
    key: K,
    value: V,
    marker: &'a [u8; N],
}

#[derive(VersionBoundary)]
#[thisversion(model = Application<'a, K, V, N>)]
struct Snapshot<'a, K, V: 'a, const N: usize>
where
    K: AsRef<str>,
{
    key: K,
    value: V,
    marker: &'a [u8; N],
}

#[derive(VersionFamily)]
enum Current<'a, K, V, const N: usize>
where
    K: AsRef<str>,
{
    Current(Snapshot<'a, K, V, N>),
}

#[test]
fn generic_model_boundaries_preserve_all_generic_parameter_kinds() {
    let application = Application {
        key: Name(String::from("example")),
        value: String::from("payload"),
        marker: b"abc",
    };

    let current: Current<'_, Name, String, 3> = application.into_schema();
    let recovered: Application<'_, Name, String, 3> = current.into_model();

    // Check: owned generic values move through the boundary:
    assert_eq!(recovered.key.as_ref(), "example");
    assert_eq!(recovered.value, "payload");

    // Check lifetimes and const generics survive the round trip:
    assert_eq!(recovered.marker, b"abc");
}

//============================================================================//
// QUALIFIED GENERIC MODEL PATHS                                              //
//============================================================================//

mod qualified {
    pub struct Model<T> {
        pub value: T,
    }
}

#[derive(VersionBoundary)]
#[thisversion(model = qualified::Model<String>)]
struct QualifiedSnapshot {
    value: String,
}

#[test]
fn qualified_generic_model_paths_round_trip() {
    let model = qualified::Model {
        value: "qualified".to_owned(),
    };

    let schema: QualifiedSnapshot = model.into_schema();
    let recovered: qualified::Model<String> = schema.into_model();

    assert_eq!(recovered.value, "qualified");
}

//============================================================================//
// GENERIC-NAME COLLISIONS                                                    //
//============================================================================//

struct CollisionModel<V> {
    value: V,
}

#[derive(VersionBoundary)]
#[thisversion(model = CollisionModel<V>)]
struct CollisionSnapshot<V> {
    // Deliberately named `V`: the derive may need its own type parameter for
    // the schema representation and must not collide with existing generics.
    value: V,
}

#[derive(VersionFamily)]
enum CollisionHistory<V> {
    Current(CollisionSnapshot<V>),
}

#[test]
fn generated_generics_do_not_collide_with_user_generic_names() {
    let model = CollisionModel {
        value: String::from("value"),
    };

    let history: CollisionHistory<String> = model.into_schema();
    let recovered: CollisionModel<String> = history.into_model();

    assert_eq!(recovered.value, "value");
}
