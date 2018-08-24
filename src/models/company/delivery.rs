use schema::delivery_from;
use schema::delivery_to;

use stq_static_resources::DeliveryCompany;

use errors::Error;
use failure::Error as FailureError;
use failure::Fail;
use serde_json;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "delivery_from"]
pub struct DeliveryFrom {
    pub id: i32,
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct NewDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct UpdateDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_from"]
pub struct OldDeliveryFrom {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub restriction_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdditionalInfo {
    data: String,
}

#[derive(Serialize, Deserialize, Associations, Queryable, Debug)]
#[table_name = "delivery_to"]
pub struct DeliveryToRaw {
    pub id: i32,
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeliveryTo {
    pub id: i32,
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<AdditionalInfo>,
}

impl DeliveryTo {
    pub fn from_raw(delivery: DeliveryToRaw) -> Result<Self, FailureError> {
        let additional_info = match delivery.additional_info {
            Some(info) => {
                serde_json::from_value(info).map_err(|e| e.context("Can not parse additional_info from db").context(Error::Parse))?
            }
            None => None,
        };

        Ok(Self {
            id: delivery.id,
            company_id: delivery.company_id,
            country: delivery.country,
            additional_info,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct NewDeliveryToRaw {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<AdditionalInfo>,
}

impl NewDeliveryTo {
    pub fn to_raw(self) -> Result<NewDeliveryToRaw, FailureError> {
        let additional_info = match self.additional_info {
            Some(info) => {
                Some(serde_json::to_value(info).map_err(|e| e.context("Can not parse additional_info from value").context(Error::Parse))?)
            }
            None => None,
        };

        Ok(NewDeliveryToRaw {
            company_id: self.company_id,
            country: self.country,
            additional_info,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct UpdateDeliveryToRaw {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
    pub additional_info: Option<AdditionalInfo>,
}

impl UpdateDeliveryTo {
    pub fn to_raw(self) -> Result<UpdateDeliveryToRaw, FailureError> {
        let additional_info = match self.additional_info {
            Some(info) => {
                Some(serde_json::to_value(info).map_err(|e| e.context("Can not parse additional_info from value").context(Error::Parse))?)
            }
            None => None,
        };

        Ok(UpdateDeliveryToRaw {
            company_id: self.company_id,
            country: self.country,
            additional_info,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "delivery_to"]
pub struct OldDeliveryTo {
    pub company_id: DeliveryCompany,
    pub country: String,
}
