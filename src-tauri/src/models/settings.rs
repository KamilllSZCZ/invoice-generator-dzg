use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub company: SellerProfile,
    pub document_defaults: DocumentDefaults,
    pub numbering: Vec<NumberingSeriesSettings>,
    pub appearance: DocumentAppearanceSettings,
    pub preferences: AppPreferences,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SellerProfile {
    pub name: Option<String>,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub tax_id: Option<String>,
    pub registration_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub bank_account: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSellerSnapshot {
    pub schema_version: u32,
    pub common: SellerProfile,
    pub document_extension: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentDefaults {
    pub document_type: DocumentTypeKey,
    pub currency: Option<String>,
    pub payment_method: Option<String>,
    pub payment_term_days: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumberingSeriesSettings {
    pub document_type: DocumentTypeKey,
    pub prefix: String,
    pub format: String,
    pub reset_policy: Option<String>,
    pub next_counter: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentAppearanceSettings {
    pub template: Option<String>,
    pub accent_color: Option<String>,
    pub show_logo: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppPreferences {
    pub language: Option<String>,
    pub theme: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentTypeKey(String);

impl DocumentTypeKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn invoice() -> Self {
        Self("invoice".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DocumentTypeKey {
    fn default() -> Self {
        Self::invoice()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, DocumentSellerSnapshot, DocumentTypeKey, SellerProfile};
    use serde_json::json;

    #[test]
    fn seller_profile_allows_unregistered_optional_fields() {
        let seller = SellerProfile::default();
        let serialized = serde_json::to_value(seller).unwrap();
        assert_eq!(serialized["tax_id"], serde_json::Value::Null);
        assert_eq!(serialized["registration_id"], serde_json::Value::Null);

        let defaults = AppSettings::default();
        let settings = serde_json::to_value(defaults).unwrap();
        assert!(settings["company"]["tax_id"].is_null());
        assert!(settings["company"]["registration_id"].is_null());
    }

    #[test]
    fn document_snapshot_supports_versioned_type_extension() {
        let snapshot = DocumentSellerSnapshot {
            schema_version: 2,
            common: SellerProfile::default(),
            document_extension: json!({ "custom_registration": "local value" }),
        };
        let value = serde_json::to_value(snapshot).unwrap();
        assert_eq!(value["schema_version"], 2);
        assert_eq!(
            value["document_extension"]["custom_registration"],
            "local value"
        );

        let key = DocumentTypeKey::invoice();
        assert_eq!(serde_json::to_string(&key).unwrap(), "\"invoice\"");
        assert_eq!(key.as_str(), "invoice");
    }
}
