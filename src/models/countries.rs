//! Models contains all structures that are used in different
//! modules of the app
//! EAV model countries
use failure::Error as FailureError;
use failure::Fail;
use serde_json;
use validator::Validate;

use stq_static_resources::{Language, Translation};
use stq_types::CountryLabel;

use errors::Error;
use models::validation_rules::*;
use schema::countries;

/// RawCountry is an object stored in PG, used only for Country tree creation,
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone)]
#[table_name = "countries"]
pub struct RawCountry {
    pub label: CountryLabel,
    pub name: serde_json::Value,
    pub parent_label: Option<CountryLabel>,
    pub level: i32,
}

/// Payload for creating countries
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "countries"]
pub struct NewCountry {
    pub label: CountryLabel,
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub parent_label: Option<CountryLabel>,
    #[validate(range(min = "1", max = "3"))]
    pub level: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Country {
    pub label: CountryLabel,
    pub name: Vec<Translation>,
    pub level: i32,
    pub parent_label: Option<CountryLabel>,
    pub children: Vec<Country>,
}

impl Default for Country {
    fn default() -> Self {
        Self {
            label: "root".to_string().into(),
            name: vec![Translation::new(Language::En, "root".to_string())],
            children: vec![],
            level: 0,
            parent_label: None,
        }
    }
}

impl RawCountry {
    pub fn to_country(&self) -> Result<Country, FailureError> {
        let name = serde_json::from_value::<Vec<Translation>>(self.name.clone())
            .map_err(|e| e.context("Can not parse name from db").context(Error::Parse))?;
        Ok(Country {
            label: self.label.clone(),
            name,
            children: vec![],
            parent_label: self.parent_label.clone(),
            level: self.level,
        })
    }
}
