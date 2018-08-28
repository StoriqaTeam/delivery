use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use errors::Error;
use schema::packages;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeliveriesTo {
    country_labels: String,
}

#[derive(Serialize, Deserialize, Associations, Queryable, Debug, QueryableByName)]
#[table_name = "packages"]
pub struct PackagesRaw {
    pub id: i32,
    pub name: String,
    pub max_size: f64,
    pub min_size: f64,
    pub max_weight: f64,
    pub min_weight: f64,
    pub deliveries_to: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Packages {
    pub id: i32,
    pub name: String,
    pub max_size: f64,
    pub min_size: f64,
    pub max_weight: f64,
    pub min_weight: f64,
    pub deliveries_to: Vec<DeliveriesTo>,
}

impl PackagesRaw {
    pub fn to_packages(self) -> Result<Packages, FailureError> {
        let deliveries_to =
            serde_json::from_value(self.deliveries_to).map_err(|e| e.context("Can not parse deliveries_to from db").context(Error::Parse))?;

        Ok(Packages {
            id: self.id,
            name: self.name,
            max_size: self.max_size,
            min_size: self.min_size,
            max_weight: self.max_weight,
            min_weight: self.min_weight,
            deliveries_to,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "packages"]
pub struct NewPackagesRaw {
    pub name: String,
    pub max_size: f64,
    pub min_size: f64,
    pub max_weight: f64,
    pub min_weight: f64,
    pub deliveries_to: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewPackages {
    pub name: String,
    pub max_size: f64,
    pub min_size: f64,
    pub max_weight: f64,
    pub min_weight: f64,
    pub deliveries_to: Vec<DeliveriesTo>,
}

impl NewPackages {
    pub fn to_raw(self) -> Result<NewPackagesRaw, FailureError> {
        let deliveries_to = serde_json::to_value(self.deliveries_to)
            .map_err(|e| e.context("Can not parse deliveries_to from value").context(Error::Parse))?;

        Ok(NewPackagesRaw {
            name: self.name,
            max_size: self.max_size,
            min_size: self.min_size,
            max_weight: self.max_weight,
            min_weight: self.min_weight,
            deliveries_to,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "packages"]
pub struct UpdatePackagesRaw {
    pub name: Option<String>,
    pub max_size: Option<f64>,
    pub min_size: Option<f64>,
    pub max_weight: Option<f64>,
    pub min_weight: Option<f64>,
    pub deliveries_to: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePackages {
    pub name: Option<String>,
    pub max_size: Option<f64>,
    pub min_size: Option<f64>,
    pub max_weight: Option<f64>,
    pub min_weight: Option<f64>,
    pub deliveries_to: Option<Vec<DeliveriesTo>>,
}

impl UpdatePackages {
    pub fn to_raw(self) -> Result<UpdatePackagesRaw, FailureError> {
        let deliveries_to = match self.deliveries_to {
            Some(info) => {
                Some(serde_json::to_value(info).map_err(|e| e.context("Can not parse deliveries_to from value").context(Error::Parse))?)
            }
            None => None,
        };

        Ok(UpdatePackagesRaw {
            name: self.name,
            max_size: self.max_size,
            min_size: self.min_size,
            max_weight: self.max_weight,
            min_weight: self.min_weight,
            deliveries_to,
        })
    }
}
