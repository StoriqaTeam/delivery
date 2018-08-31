//! Repos contains all info about working with countries
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{CountryLabel, UserId};

use models::authorization::*;
use models::{Country, NewCountry, RawCountry};
use repos::acl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;
use schema::countries::dsl::*;

pub mod cache;

pub use self::cache::*;

/// Countries repository, responsible for handling countries
pub struct CountriesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Country>>,
    pub cache: CountryCacheImpl,
}

pub trait CountriesRepo {
    /// Find specific country by label
    fn find(&self, label_arg: CountryLabel) -> RepoResult<Option<Country>>;

    /// Creates new country
    fn create(&self, payload: NewCountry) -> RepoResult<Country>;

    /// Returns all countries as a tree
    fn get_all(&self) -> RepoResult<Country>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CountriesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Country>>, cache: CountryCacheImpl) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CountriesRepo for CountriesRepoImpl<'a, T> {
    /// Find specific country by label
    fn find(&self, label_arg: CountryLabel) -> RepoResult<Option<Country>> {
        debug!("Find in countries with label {}.", label_arg);
        acl::check(&*self.acl, Resource::Countries, Action::Read, self, None)?;
        self.get_all().map(|root| get_country(&root, label_arg))
    }

    /// Creates new country
    fn create(&self, payload: NewCountry) -> RepoResult<Country> {
        debug!("Create new country {:?}.", payload);
        self.cache.clear();
        let query = diesel::insert_into(countries).values(&payload);
        query
            .get_result::<RawCountry>(self.db_conn)
            .map_err(From::from)
            .and_then(|created_country| created_country.to_country())
            .and_then(|country| acl::check(&*self.acl, Resource::Countries, Action::Create, self, Some(&country)).and_then(|_| Ok(country)))
            .map_err(|e: FailureError| e.context(format!("Create new country: {:?} error occured", payload)).into())
    }

    fn get_all(&self) -> RepoResult<Country> {
        if let Some(country) = self.cache.get() {
            debug!("Get all countries from cache request.");
            Ok(country)
        } else {
            debug!("Get all countries from db request.");
            acl::check(&*self.acl, Resource::Countries, Action::Read, self, None)
                .and_then(|_| {
                    let countries_ = countries.load::<RawCountry>(self.db_conn)?;
                    let mut root = Country::default();
                    let children = create_tree(&countries_, None)?;
                    root.children = children;
                    self.cache.set(root.clone());
                    Ok(root)
                })
                .map_err(|e: FailureError| e.context("Get all countries error occured").into())
        }
    }
}

fn create_tree(countries_: &[RawCountry], parent_label_arg: Option<CountryLabel>) -> RepoResult<Vec<Country>> {
    let mut branch = vec![];
    for country in countries_ {
        if country.parent_label == parent_label_arg {
            let childs = create_tree(countries_, Some(country.label.clone()))?;
            let mut country_tree: Country = country.to_country()?;
            country_tree.children = childs;
            branch.push(country_tree);
        }
    }
    Ok(branch)
}

pub fn remove_unused_countries(mut country: Country, used_countries_labels: &[CountryLabel], stack_level: i32) -> Country {
    let mut children = vec![];
    for country_child in country.children {
        if stack_level == 0 {
            if used_countries_labels.iter().any(|used_label| country_child.label == *used_label) {
                children.push(country_child);
            }
        } else {
            let new_country = remove_unused_countries(country_child, used_countries_labels, stack_level - 1);
            if !new_country.children.is_empty() {
                children.push(new_country);
            }
        }
    }
    country.children = children;
    country
}

pub fn clear_child_countries(mut country: Country, stack_level: i32) -> Country {
    if stack_level == 0 {
        country.children.clear();
    } else {
        let mut countries_ = vec![];
        for country_child in country.children {
            let new_country = clear_child_countries(country_child, stack_level - 1);
            countries_.push(new_country);
        }
        country.children = countries_;
    }
    country
}

pub fn get_parent_country(country: &Country, child_label: CountryLabel, stack_level: i32) -> Option<Country> {
    if stack_level != 0 {
        country
            .children
            .iter()
            .find(|country_child| get_parent_country(country_child, child_label.clone(), stack_level - 1).is_some())
            .and_then(|_| Some(country.clone()))
    } else if country.label == child_label {
        Some(country.clone())
    } else {
        None
    }
}

pub fn get_country(country: &Country, country_label: CountryLabel) -> Option<Country> {
    if country.label == country_label {
        Some(country.clone())
    } else {
        country
            .children
            .iter()
            .filter_map(|country_child| get_country(country_child, country_label.clone()))
            .next()
    }
}

pub fn get_all_children_till_the_end(country: Country) -> Vec<Country> {
    if country.children.is_empty() {
        vec![country]
    } else {
        let mut klabels = vec![];
        for country_child in country.children {
            let mut children_klabels = get_all_children_till_the_end(country_child);
            klabels.append(&mut children_klabels);
        }
        klabels
    }
}

pub fn contains_country_label(country: &Country, country_label: &CountryLabel) -> bool {
    if country.label == country_label.clone() {
        true
    } else {
        country
            .children
            .iter()
            .any(|country_child| contains_country_label(country_child, country_label))
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Country>
    for CountriesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_label: UserId, scope: &Scope, _obj: Option<&Country>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::*;
    use stq_types::CountryLabel;

    fn create_mock_countries() -> Country {
        let country_3 = Country {
            label: "RUS".to_string().into(),
            name: vec![],
            children: vec![],
            level: 2,
            parent_label: Some("EEE".to_string().into()),
        };
        let country_2 = Country {
            label: "EEE".to_string().into(),
            name: vec![],
            children: vec![country_3],
            level: 1,
            parent_label: Some(ALL_COUNTRIES.clone()),
        };
        Country {
            children: vec![country_2],
            ..Default::default()
        }
    }

    #[test]
    fn test_parent_countries() {
        let country = create_mock_countries();
        let child_label = CountryLabel("rus".to_string());
        let new_country = country
            .children
            .into_iter()
            .find(|country_child| get_parent_country(&country_child, child_label.clone(), 1).is_some())
            .unwrap();
        assert_eq!(new_country.label, ALL_COUNTRIES.clone());
    }

    #[test]
    fn test_get_country() {
        let country = create_mock_countries();
        let child_label = CountryLabel("RUS".to_string());
        let new_country = get_country(&country, child_label.clone()).unwrap();
        assert_eq!(new_country.label, child_label.clone().into());
    }
}
