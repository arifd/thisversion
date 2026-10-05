use thisversion::VersionFamily;
use thisversion::traits::{IntoModel, IntoSchema};

//============================================================================//
// CANONICAL                                                                  //
//============================================================================//

/// The canonical contact model used by the application.
///
/// The application supports contacts from two independent sources: Google
/// Contacts and Outlook Contacts. Each source has its own schema history and
/// evolves independently.
///
/// Both ultimately represent the same application-level concept, so their
/// latest versions can be converted into this common model.
///
/// The contact schemas in this example are intentionally simplified and do not
/// model the actual Google or Outlook APIs.
#[derive(Debug, PartialEq, Eq)]
pub struct Contact {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

//============================================================================//
// GOOGLE CONTACTS                                                            //
//============================================================================//

/// Every supported schema version of a Google contact.
///
/// Google contacts have their own migration history, independent of the Outlook
/// contact family below.
#[derive(VersionFamily)]
pub enum GoogleContactVersions {
    V1(GoogleContactV1),
    V2(GoogleContactV2),
    V3(GoogleContactV3),
}

/// The first Google contact schema.
///
/// Contacts initially consisted only of a name and email address.
pub struct GoogleContactV1 {
    pub name: String,
    pub email: String,
}

/// The second Google contact schema adds an optional phone number.
pub struct GoogleContactV2 {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

/// Existing contacts have no phone number when migrated to V2.
impl From<GoogleContactV1> for GoogleContactV2 {
    fn from(prev: GoogleContactV1) -> Self {
        Self {
            name: prev.name,
            email: prev.email,
            phone: None,
        }
    }
}

/// The latest Google contact schema renames `email` to `email_address`.
pub struct GoogleContactV3 {
    pub name: String,
    pub email_address: String,
    pub phone: Option<String>,
}

/// Migrating from V2 preserves the contact while adopting the latest field
/// name.
impl From<GoogleContactV2> for GoogleContactV3 {
    fn from(prev: GoogleContactV2) -> Self {
        Self {
            name: prev.name,
            email_address: prev.email,
            phone: prev.phone,
        }
    }
}

/// The derive requires matching field names, so `email_address` needs an
/// explicit mapping to the canonical model's `email` field.
impl IntoModel for GoogleContactV3 {
    type Model = Contact;

    fn into_model(self) -> Self::Model {
        Contact {
            name: self.name,
            email: self.email_address,
            phone: self.phone,
        }
    }
}

/// Define how the canonical contact model is converted back into the latest
/// Google contact schema.
impl IntoSchema<GoogleContactVersions> for Contact {
    fn into_schema(self) -> GoogleContactVersions {
        GoogleContactVersions::V3(GoogleContactV3 {
            name: self.name,
            email_address: self.email,
            phone: self.phone,
        })
    }
}

//============================================================================//
// OUTLOOK CONTACTS                                                           //
//============================================================================//

/// Every supported schema version of an Outlook contact.
///
/// Outlook contacts have their own migration history, independent of the Google
/// contact family above.
#[derive(VersionFamily)]
pub enum OutlookContactVersions {
    V1(OutlookContactV1),
    V2(OutlookContactV2),
    V3(OutlookContactV3),
}

/// The first Outlook contact schema stores the contact's name as separate
/// components.
pub struct OutlookContactV1 {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

/// The second Outlook contact schema adds an optional phone number.
pub struct OutlookContactV2 {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
}

/// Existing contacts have no phone number when migrated to V2.
impl From<OutlookContactV1> for OutlookContactV2 {
    fn from(prev: OutlookContactV1) -> Self {
        Self {
            first_name: prev.first_name,
            last_name: prev.last_name,
            email: prev.email,
            phone: None,
        }
    }
}

/// The latest Outlook contact schema introduces a display name.
///
/// Outlook still retains the individual name components, but the display name
/// is what the application uses when converting the contact into its canonical
/// representation.
pub struct OutlookContactV3 {
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub email: String,
    pub phone: Option<String>,
}

/// Migrating from V2 derives the display name from the existing name
/// components.
impl From<OutlookContactV2> for OutlookContactV3 {
    fn from(prev: OutlookContactV2) -> Self {
        let display_name = format!("{} {}", prev.first_name, prev.last_name);

        Self {
            first_name: prev.first_name,
            last_name: prev.last_name,
            display_name,
            email: prev.email,
            phone: prev.phone,
        }
    }
}

/// The derive requires matching field names, so `display_name` needs an
/// explicit mapping to the canonical model's `name` field.
///
/// This conversion is intentionally one-way. The canonical model does not
/// retain Outlook's separate first and last names, so converting a `Contact`
/// back into an `OutlookContactV3` would require inventing information.
impl IntoModel for OutlookContactV3 {
    type Model = Contact;

    fn into_model(self) -> Self::Model {
        Contact {
            name: self.display_name,
            email: self.email,
            phone: self.phone,
        }
    }
}

//============================================================================//
// APPLICATION                                                                //
//============================================================================//

fn main() {
    //========================================================================//
    // GOOGLE CONTACT                                                         //
    //========================================================================//

    // Imagine this value came from an earlier version of the Google contact
    // model.
    let google = GoogleContactVersions::V1(GoogleContactV1 {
        name: "Ada Lovelace".to_owned(),
        email: "ada@example.com".to_owned(),
    });

    // The value first follows the Google contact family's own migration
    // history to `GoogleContactV3`, then crosses into `Contact`.
    let from_google = google.into_model();

    assert_eq!(from_google.name, "Ada Lovelace");
    assert_eq!(from_google.email, "ada@example.com");
    assert_eq!(from_google.phone, None);

    //========================================================================//
    // OUTLOOK CONTACT                                                        //
    //========================================================================//

    // The same logical contact could instead come from an earlier version of
    // the independently evolving Outlook contact model.
    let outlook = OutlookContactVersions::V1(OutlookContactV1 {
        first_name: "Ada".to_owned(),
        last_name: "Lovelace".to_owned(),
        email: "ada@example.com".to_owned(),
    });

    // This value follows the Outlook contact family's own migration history
    // before crossing into the same canonical application model.
    let from_outlook = outlook.into_model();

    assert_eq!(from_outlook.name, "Ada Lovelace");
    assert_eq!(from_outlook.email, "ada@example.com");
    assert_eq!(from_outlook.phone, None);

    //========================================================================//
    // CONVERGENCE                                                            //
    //========================================================================//

    // The two contact families have independent schema histories and no
    // migration edges between them. They converge only at the application's
    // canonical model.
    assert_eq!(from_google, from_outlook);

    //========================================================================//
    // DIRECTIONAL CONVERSION                                                 //
    //========================================================================//

    // Convergence does not imply that every conversion must be reversible.
    //
    // `Contact` implements `IntoSchema<GoogleContactVersions>`, so a
    // canonical contact can be converted into the latest Google schema and
    // wrapped in its version family.
    let contact = Contact {
        name: "Ada Lovelace".to_owned(),
        email: "ada@example.com".to_owned(),
        phone: None,
    };

    let google: GoogleContactVersions = contact.into_schema();

    assert!(matches!(google, GoogleContactVersions::V3(_)));

    // There is deliberately no `IntoSchema<OutlookContactVersions>` for
    // `Contact`. The canonical model does not retain the separate first and
    // last names required to reconstruct an Outlook contact faithfully.
}
