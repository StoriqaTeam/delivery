//! Repos contains all info about working with countries
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::Bool;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{self, Alpha3, CountryLabel, UserId};

use models::authorization::*;
use models::{Country, NewCountry, RawCountry};
use repos::acl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;
use schema::countries::dsl::*;

pub mod cache;

pub use self::cache::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CountrySearch {
    Label(CountryLabel),
    Alpha2(stq_types::Alpha2),
    Alpha3(stq_types::Alpha3),
    Numeric(i32),
}

/// Countries repository, responsible for handling countries
pub struct CountriesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Country>>,
    pub cache: CountryCacheImpl,
}

pub trait CountriesRepo {
    /// Returns tree country
    fn find(&self, label_arg: Alpha3) -> RepoResult<Option<Country>>;

    /// Returns country by codes
    fn find_by(&self, search: CountrySearch) -> RepoResult<Option<Country>>;

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
    fn find(&self, arg: Alpha3) -> RepoResult<Option<Country>> {
        debug!("Find in countries with aplha3 {}.", arg);
        acl::check(&*self.acl, Resource::Countries, Action::Read, self, None)?;
        self.get_all().map(|root| get_country(&root, &arg))
    }

    fn find_by(&self, search: CountrySearch) -> RepoResult<Option<Country>> {
        debug!("Get countries by search: {:?}.", search);

        let search_exp: Box<BoxableExpression<countries, _, SqlType = Bool>> = match search.clone() {
            CountrySearch::Label(value) => Box::new(label.eq(value)),
            CountrySearch::Alpha2(value) => Box::new(alpha2.eq(value)),
            CountrySearch::Alpha3(value) => Box::new(alpha3.eq(value)),
            CountrySearch::Numeric(value) => Box::new(numeric.eq(value)),
        };

        let query = countries.filter(search_exp);
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|raw_country: Option<RawCountry>| match raw_country {
                Some(raw_country) => {
                    let country: Country = raw_country.into();
                    acl::check(&*self.acl, Resource::Countries, Action::Read, self, Some(&country))?;

                    Ok(Some(country))
                }
                None => Ok(None),
            })
            .map_err(|e: FailureError| e.context(format!("Get countries by search: {:?}.", search)).into())
    }

    /// Creates new country
    fn create(&self, payload: NewCountry) -> RepoResult<Country> {
        debug!("Create new country {:?}.", payload);
        self.cache.clear();
        let query = diesel::insert_into(countries).values(&payload);
        query
            .get_result::<RawCountry>(self.db_conn)
            .map_err(From::from)
            .map(From::from)
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
                    let tree = create_tree(&countries_, None)?;
                    let root = tree
                        .into_iter()
                        .nth(0)
                        .ok_or_else(|| format_err!("Could not create countries tree"))?;
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
            let mut country_tree: Country = country.into();
            country_tree.children = childs;
            branch.push(country_tree);
        }
    }
    Ok(branch)
}

pub fn remove_unused_countries(mut country: Country, used_countries_codes: &[Alpha3], stack_level: i32) -> Country {
    let mut children = vec![];
    for country_child in country.children {
        if stack_level == 0 {
            if used_countries_codes.iter().any(|used_code| country_child.alpha3 == *used_code) {
                children.push(country_child);
            }
        } else {
            let new_country = remove_unused_countries(country_child, used_countries_codes, stack_level - 1);
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

pub fn get_parent_country(country: &Country, child_code: &Alpha3, stack_level: i32) -> Option<Country> {
    if stack_level != 0 {
        country
            .children
            .iter()
            .find(|country_child| get_parent_country(country_child, child_code, stack_level - 1).is_some())
            .and_then(|_| Some(country.clone()))
    } else if country.alpha3 == *child_code {
        Some(country.clone())
    } else {
        None
    }
}

pub fn get_country(country: &Country, country_id: &Alpha3) -> Option<Country> {
    if country.alpha3 == *country_id {
        Some(country.clone())
    } else {
        country
            .children
            .iter()
            .filter_map(|country_child| get_country(country_child, country_id))
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

pub fn get_all_parent_codes(country: &Country, searched_country_id: &Alpha3, codes: &mut Vec<Alpha3>) {
    if country.alpha3 == *searched_country_id {
        codes.push(country.alpha3.clone())
    } else {
        for child in &country.children {
            let old_len = codes.len();
            get_all_parent_codes(child, searched_country_id, codes);
            if codes.len() > old_len {
                codes.push(country.alpha3.clone());
                break;
            }
        }
    }
}

pub fn set_selected(country: &mut Country, selected_codes: &[Alpha3]) {
    if selected_codes.iter().any(|country_code| &country.alpha3 == country_code) {
        set_selected_till_end(country);
    } else {
        for child in &mut country.children {
            set_selected(child, selected_codes);
        }
    }
}

pub fn get_selected(country: &Country, codes: &mut Vec<Alpha3>) {
    if country.is_selected {
        codes.push(country.alpha3.clone())
    } else {
        for child in &country.children {
            get_selected(child, codes);
        }
    }
}

pub fn set_selected_till_end(country: &mut Country) {
    country.is_selected = true;
    for child in &mut country.children {
        set_selected_till_end(child);
    }
}

pub fn contains_country_code(country: &Country, country_code: &Alpha3) -> bool {
    if country.alpha3 == country_code.clone() {
        true
    } else {
        country
            .children
            .iter()
            .any(|country_child| contains_country_code(country_child, country_code))
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
    use stq_types::{Alpha2, Alpha3};

    fn create_mock_countries() -> Country {
        let country_3 = Country {
            label: "Russia".to_string().into(),
            children: vec![],
            level: 2,
            parent_label: Some("Europe".to_string().into()),
            alpha2: Alpha2("RU".to_string()),
            alpha3: Alpha3("RUS".to_string()),
            numeric: 0,
            is_selected: false,
        };
        let country_2 = Country {
            label: "Europe".to_string().into(),
            children: vec![country_3],
            level: 1,
            parent_label: Some("All".to_string().into()),
            alpha2: Alpha2("".to_string()),
            alpha3: Alpha3("XEU".to_string()),
            numeric: 0,
            is_selected: false,
        };
        Country {
            label: "All".to_string().into(),
            level: 0,
            parent_label: None,
            children: vec![country_2],
            alpha2: Alpha2("".to_string()),
            alpha3: Alpha3("XAL".to_string()),
            numeric: 0,
            is_selected: false,
        }
    }

    #[test]
    fn test_parent_countries() {
        let country = create_mock_countries();
        let child_code = Alpha3("RUS".to_string());
        let new_country = country
            .children
            .into_iter()
            .find(|country_child| get_parent_country(&country_child, &child_code, 1).is_some())
            .unwrap();
        assert_eq!(new_country.label, "Europe".to_string().into());
    }

    #[test]
    fn test_get_country() {
        let country = create_mock_countries();
        let child_code = Alpha3("RUS".to_string());
        let new_country = get_country(&country, &child_code).unwrap();
        assert_eq!(new_country.alpha3, child_code.clone().into());
    }
}
