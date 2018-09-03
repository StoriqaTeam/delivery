//! Models contains all structures that are used in different
//! modules of the app
//! EAV model countries
use validator::Validate;

use stq_types::CountryLabel;

use schema::countries;

/// RawCountry is an object stored in PG, used only for Country tree creation,
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone)]
#[table_name = "countries"]
pub struct RawCountry {
    pub label: CountryLabel,
    pub parent_label: Option<CountryLabel>,
    pub level: i32,
    pub alpha2: String,
    pub alpha3: String,
    pub numeric: i32,
}

/// Payload for creating countries
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "countries"]
pub struct NewCountry {
    pub label: CountryLabel,
    pub parent_label: Option<CountryLabel>,
    #[validate(range(min = "1", max = "2"))]
    pub level: i32,
    pub alpha2: String,
    pub alpha3: String,
    pub numeric: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Country {
    pub label: CountryLabel,
    pub level: i32,
    pub parent_label: Option<CountryLabel>,
    pub alpha2: String,
    pub alpha3: String,
    pub numeric: i32,
    pub children: Vec<Country>,
    pub is_selected: bool,
}

impl From<RawCountry> for Country {
    fn from(raw: RawCountry) -> Self {
        Self {
            label: raw.label.clone(),
            children: vec![],
            parent_label: raw.parent_label.clone(),
            level: raw.level,
            alpha2: raw.alpha2,
            alpha3: raw.alpha3,
            numeric: raw.numeric,
            is_selected: false,
        }
    }
}

impl<'a> From<&'a RawCountry> for Country {
    fn from(raw: &RawCountry) -> Self {
        Self {
            label: raw.label.clone(),
            children: vec![],
            parent_label: raw.parent_label.clone(),
            level: raw.level,
            alpha2: raw.alpha2.clone(),
            alpha3: raw.alpha3.clone(),
            numeric: raw.numeric,
            is_selected: false,
        }
    }
}
