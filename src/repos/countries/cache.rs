//! CountryCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};

use models::Country;

#[derive(Clone, Default)]
pub struct CountryCacheImpl {
    inner: Arc<Mutex<Option<Country>>>,
}

impl CountryCacheImpl {
    pub fn get(&self) -> Option<Country> {
        let country = self.inner.lock().unwrap();
        country.clone()
    }

    pub fn clear(&self) {
        let mut country = self.inner.lock().unwrap();
        *country = None;
    }

    pub fn is_some(&self) -> bool {
        let country = self.inner.lock().unwrap();
        country.is_some()
    }

    pub fn set(&self, cat: Country) {
        let mut country = self.inner.lock().unwrap();
        *country = Some(cat);
    }
}
