use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use stq_types::{Alpha3, PackageId};

use errors::Error;
use models::{Country, ShipmentMeasurements};
use repos::countries::create_tree_used_countries;
use schema::packages;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum MeasurementsOutOfRange {
    VolumeOutOfRange {
        volume_cubic_cm: u32,
        min_volume_cubic_cm: u32,
        max_volume_cubic_cm: u32,
    },
    WeightOutOfRange {
        weight_g: u32,
        min_weight_g: u32,
        max_weight_g: u32,
    },
    VolumeAndWeightOutOfRange {
        volume_cubic_cm: u32,
        min_volume_cubic_cm: u32,
        max_volume_cubic_cm: u32,
        weight_g: u32,
        min_weight_g: u32,
        max_weight_g: u32,
    },
}

#[derive(Serialize, Deserialize, Associations, Queryable, Debug, QueryableByName)]
#[table_name = "packages"]
pub struct PackagesRaw {
    pub id: PackageId,
    pub name: String,
    pub max_size: i32,
    pub min_size: i32,
    pub max_weight: i32,
    pub min_weight: i32,
    pub deliveries_to: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Packages {
    pub id: PackageId,
    pub name: String,
    pub max_size: u32,
    pub min_size: u32,
    pub max_weight: u32,
    pub min_weight: u32,
    pub deliveries_to: Vec<Country>,
}

impl Packages {
    pub fn within_limits(&self, measurements: ShipmentMeasurements) -> Result<(), MeasurementsOutOfRange> {
        let Packages {
            max_size,
            min_size,
            max_weight,
            min_weight,
            ..
        } = self;

        let (max_size, min_size, max_weight, min_weight) = (max_size.clone(), min_size.clone(), max_weight.clone(), min_weight.clone());

        let ShipmentMeasurements { volume_cubic_cm, weight_g } = measurements;

        let volume_out_of_range = !(min_size <= volume_cubic_cm && volume_cubic_cm <= max_size);
        let weight_out_of_range = !(min_weight <= weight_g && weight_g <= max_weight);

        match (volume_out_of_range, weight_out_of_range) {
            (false, false) => Ok(()),
            (true, false) => Err(MeasurementsOutOfRange::VolumeOutOfRange {
                volume_cubic_cm,
                min_volume_cubic_cm: min_size,
                max_volume_cubic_cm: max_size,
            }),
            (false, true) => Err(MeasurementsOutOfRange::WeightOutOfRange {
                weight_g,
                min_weight_g: min_weight,
                max_weight_g: max_weight,
            }),
            (true, true) => Err(MeasurementsOutOfRange::VolumeAndWeightOutOfRange {
                volume_cubic_cm,
                min_volume_cubic_cm: min_size,
                max_volume_cubic_cm: max_size,
                weight_g,
                min_weight_g: min_weight,
                max_weight_g: max_weight,
            }),
        }
    }
}

impl PackagesRaw {
    pub fn to_packages(self, countries_arg: &Country) -> Result<Packages, FailureError> {
        let used_codes: Vec<Alpha3> =
            serde_json::from_value(self.deliveries_to).map_err(|e| e.context("Can not parse deliveries_to from db"))?;
        let deliveries_to = create_tree_used_countries(countries_arg, &used_codes);

        Ok(Packages {
            id: self.id,
            name: self.name,
            max_size: self.max_size as u32,
            min_size: self.min_size as u32,
            max_weight: self.max_weight as u32,
            min_weight: self.min_weight as u32,
            deliveries_to,
        })
    }

    pub fn get_deliveries_to(&self) -> Result<Vec<Alpha3>, FailureError> {
        let used_codes =
            serde_json::from_value(self.deliveries_to.clone()).map_err(|e| e.context("Can not parse deliveries_to from db"))?;

        Ok(used_codes)
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "packages"]
pub struct NewPackagesRaw {
    pub name: String,
    pub max_size: i32,
    pub min_size: i32,
    pub max_weight: i32,
    pub min_weight: i32,
    pub deliveries_to: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewPackages {
    pub name: String,
    pub max_size: u32,
    pub min_size: u32,
    pub max_weight: u32,
    pub min_weight: u32,
    pub deliveries_to: Vec<Alpha3>,
}

impl NewPackages {
    pub fn to_raw(self) -> Result<NewPackagesRaw, FailureError> {
        let deliveries_to = serde_json::to_value(self.deliveries_to)
            .map_err(|e| e.context("Can not parse deliveries_to from value").context(Error::Parse))?;

        Ok(NewPackagesRaw {
            name: self.name,
            max_size: self.max_size as i32,
            min_size: self.min_size as i32,
            max_weight: self.max_weight as i32,
            min_weight: self.min_weight as i32,
            deliveries_to,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "packages"]
pub struct UpdatePackagesRaw {
    pub name: Option<String>,
    pub max_size: Option<i32>,
    pub min_size: Option<i32>,
    pub max_weight: Option<i32>,
    pub min_weight: Option<i32>,
    pub deliveries_to: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePackages {
    pub name: Option<String>,
    pub max_size: Option<u32>,
    pub min_size: Option<u32>,
    pub max_weight: Option<u32>,
    pub min_weight: Option<u32>,
    pub deliveries_to: Option<Vec<Alpha3>>,
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
            max_size: self.max_size.map(|x| x as i32),
            min_size: self.min_size.map(|x| x as i32),
            max_weight: self.max_weight.map(|x| x as i32),
            min_weight: self.min_weight.map(|x| x as i32),
            deliveries_to,
        })
    }
}
