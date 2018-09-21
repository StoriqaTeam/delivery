use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use stq_types::{Alpha3, CompanyId};

use errors::Error;
use models::Country;
use repos::countries::create_tree_used_countries;
use schema::companies;

#[derive(Serialize, Deserialize, Associations, Queryable, Debug, QueryableByName)]
#[table_name = "companies"]
pub struct CompanyRaw {
    pub id: CompanyId,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub deliveries_from: serde_json::Value,
    pub logo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Company {
    pub id: CompanyId,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub deliveries_from: Vec<Country>,
    pub logo: String,
}

impl Company {
    pub fn from_raw(from: CompanyRaw, countries_arg: &Country) -> Result<Self, FailureError> {
        let used_codes: Vec<Alpha3> = serde_json::from_value(from.deliveries_from)
            .map_err(|e| e.context("Can not parse deliveries_from from db").context(Error::Parse))?;
        let deliveries_from = create_tree_used_countries(countries_arg, &used_codes);

        Ok(Self {
            id: from.id,
            name: from.name,
            label: from.label,
            description: from.description,
            deliveries_from,
            logo: from.logo,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "companies"]
pub struct NewCompanyRaw {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub deliveries_from: serde_json::Value,
    pub logo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewCompany {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub deliveries_from: Vec<Alpha3>,
    pub logo: String,
}

impl NewCompany {
    pub fn to_raw(self) -> Result<NewCompanyRaw, FailureError> {
        let Self {
            name,
            label,
            deliveries_from,
            description,
            logo,
        } = self;

        let deliveries_from = serde_json::to_value(deliveries_from)
            .map_err(|e| e.context("Can not parse deliveries_from from value").context(Error::Parse))?;

        Ok(NewCompanyRaw {
            name,
            label,
            description,
            deliveries_from,
            logo,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "companies"]
pub struct UpdateCompanyRaw {
    pub name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub deliveries_from: Option<serde_json::Value>,
    pub logo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateCompany {
    pub name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub deliveries_from: Option<Vec<Alpha3>>,
    pub logo: Option<String>,
}

impl UpdateCompany {
    pub fn to_raw(self) -> Result<UpdateCompanyRaw, FailureError> {
        let Self {
            name,
            label,
            deliveries_from,
            description,
            logo,
        } = self;

        let deliveries_from = match deliveries_from {
            Some(data) => {
                Some(serde_json::to_value(data).map_err(|e| e.context("Can not parse deliveries_from from value").context(Error::Parse))?)
            }
            None => None,
        };

        Ok(UpdateCompanyRaw {
            name,
            label,
            description,
            deliveries_from,
            logo,
        })
    }
}
