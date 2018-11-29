//! Models contains all structures that are used in different
//! modules of the app
//! EAV model countries
use validator::Validate;

use stq_types::{Alpha2, Alpha3, CountryLabel};

use models::validation_rules::*;
use schema::countries;

/// RawCountry is an object stored in PG, used only for Country tree creation,
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone)]
#[table_name = "countries"]
pub struct RawCountry {
    pub label: CountryLabel,
    pub level: i32,
    pub alpha2: Alpha2,
    pub alpha3: Alpha3,
    pub numeric: i32,
    pub parent: Option<Alpha3>,
}

/// Payload for creating countries
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "countries"]
pub struct NewCountry {
    pub label: CountryLabel,
    #[validate(range(min = "1", max = "2"))]
    pub level: i32,
    #[validate(custom = "validate_alpha2")]
    pub alpha2: Alpha2,
    #[validate(custom = "validate_alpha3")]
    pub alpha3: Alpha3,
    pub numeric: i32,
    pub parent: Option<Alpha3>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Country {
    pub label: CountryLabel,
    pub level: i32,
    pub alpha2: Alpha2,
    pub alpha3: Alpha3,
    pub numeric: i32,
    pub children: Vec<Country>,
    pub is_selected: bool,
    pub parent: Option<Alpha3>,
}

impl Country {
    pub const COUNTRY_LEVEL: i32 = 2;
}

impl From<RawCountry> for Country {
    fn from(raw: RawCountry) -> Self {
        Self {
            label: raw.label.clone(),
            children: vec![],
            level: raw.level,
            alpha2: raw.alpha2,
            alpha3: raw.alpha3,
            numeric: raw.numeric,
            is_selected: false,
            parent: raw.parent.clone(),
        }
    }
}

impl<'a> From<&'a RawCountry> for Country {
    fn from(raw: &RawCountry) -> Self {
        Self {
            label: raw.label.clone(),
            children: vec![],
            level: raw.level,
            alpha2: raw.alpha2.clone(),
            alpha3: raw.alpha3.clone(),
            numeric: raw.numeric,
            is_selected: false,
            parent: raw.parent.clone(),
        }
    }
}

pub fn get_country(country: &Country, country_id: &Alpha3) -> Option<Country> {
    if country.alpha3 == *country_id {
        Some(country.clone())
    } else {
        get_country_from_forest(country.children.iter(), country_id)
    }
}

pub fn get_country_from_forest<'a, C>(countries: C, country_id: &Alpha3) -> Option<Country>
where
    C: Iterator<Item = &'a Country>,
{
    countries.filter_map(|country| get_country(country, country_id)).next()
}

pub fn get_countries_by<P>(country: &Country, predicate: P) -> Vec<Country>
where
    P: Fn(&Country) -> bool,
{
    get_countries_by_inner(country, &predicate, Vec::default())
}

pub fn get_countries_from_forest_by<'a, C, P>(countries: C, predicate: P) -> Vec<Country>
where
    C: Iterator<Item = &'a Country>,
    P: Fn(&Country) -> bool,
{
    get_countries_from_forest_by_inner(countries, &predicate, Vec::default())
}

fn get_countries_by_inner<P>(country: &Country, predicate: &P, mut vec: Vec<Country>) -> Vec<Country>
where
    P: Fn(&Country) -> bool,
{
    if predicate(country) {
        vec.push(country.clone());
    };

    get_countries_from_forest_by_inner(country.children.iter(), predicate, vec)
}

fn get_countries_from_forest_by_inner<'a, C, P>(countries: C, predicate: &P, vec: Vec<Country>) -> Vec<Country>
where
    C: Iterator<Item = &'a Country>,
    P: Fn(&Country) -> bool,
{
    countries.fold(vec, |vec, country| get_countries_by_inner(country, predicate, vec))
}
