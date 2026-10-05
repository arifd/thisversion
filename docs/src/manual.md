# Manual conversions

Use `VersionBoundary` when matching schema and model fields can move directly or
convert through `nested`. For custom mapping or model validation, implement
`IntoModel` or `TryIntoModel` on the latest schema yourself. `VersionFamily`
still connects the history. Ordinary `From` and `TryFrom` migrations remain
part of the derive workflow described earlier.

## Custom model mapping

Here the schema's `display_name` becomes the model's `name`, which must be
nonempty when loaded. The manual conversion replaces `VersionBoundary`:

```rust
# extern crate thisversion;
use thisversion::VersionFamily;
use thisversion::traits::{IntoModel, IntoSchema, TryIntoModel};

#[derive(VersionFamily)]
#[thisversion(fallible)]
enum ContactVersions {
    V1(ContactV1),
}

struct ContactV1 {
    display_name: String,
}

struct Contact {
    name: String,
}

#[derive(Debug)]
struct EmptyName;

impl TryIntoModel for ContactV1 {
    type Model = Contact;
    type Error = EmptyName;

    fn try_into_model(self) -> Result<Contact, EmptyName> {
        if self.display_name.trim().is_empty() {
            return Err(EmptyName);
        }
        Ok(Contact { name: self.display_name })
    }
}

impl IntoSchema<ContactVersions> for Contact {
    fn into_schema(self) -> ContactVersions {
        ContactVersions::V1(ContactV1 { display_name: self.name })
    }
}
```

The validation runs for every version that reaches this latest schema,
including V1 itself. Saving is implemented separately because replacing the
derive also removes its generated output conversion. For an infallible mapping,
implement `IntoModel` instead and use an infallible family. A manual schema
needs its own `TryIntoModel` implementation if used directly with the Serde
adapter, which requires that trait.

## Several sources, one model

Another family can feed the same `Contact`. Add these definitions:

```rust
# extern crate thisversion;
# struct Contact {
#     name: String,
# }
use thisversion::traits::IntoModel;
use thisversion::VersionFamily;

#[derive(VersionFamily)]
enum DirectoryVersions {
    V1(DirectoryV1),
}

struct DirectoryV1 {
    first_name: String,
    last_name: String,
}

impl IntoModel for DirectoryV1 {
    type Model = Contact;

    fn into_model(self) -> Contact {
        Contact { name: format!("{} {}", self.first_name, self.last_name) }
    }
}
```

Assuming directory names are already valid, each family now follows its own
history into `Contact`. The directory mapping discards the original name parts,
so it deliberately has no reverse conversion. If export must preserve them,
retain them in the model or elsewhere in the application. Supporting an input
format does not require supporting it as an output format.
