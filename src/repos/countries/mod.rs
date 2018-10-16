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
    pub fn new(
        db_conn: &'a T,
        acl: Box<Acl<Resource, Action, Scope, FailureError, Country>>,
        cache: CountryCacheImpl,
    ) -> Self {
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
            }).map_err(|e: FailureError| e.context(format!("Get countries by search: {:?}.", search)).into())
    }

    /// Creates new country
    fn create(&self, payload: NewCountry) -> RepoResult<Country> {
        debug!("Create new country {:?}.", payload);
        self.cache.remove();
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
                    self.cache.set(&root);
                    Ok(root)
                }).map_err(|e: FailureError| e.context("Get all countries error occured").into())
        }
    }
}

fn create_tree(countries_: &[RawCountry], parent_arg: Option<Alpha3>) -> RepoResult<Vec<Country>> {
    let mut branch = vec![];
    for country in countries_ {
        if country.parent == parent_arg {
            let childs = create_tree(countries_, Some(country.alpha3.clone()))?;
            let mut country_tree: Country = country.into();
            country_tree.children = childs;
            branch.push(country_tree);
        }
    }
    Ok(branch)
}

pub fn create_tree_used_countries(countries_arg: &Country, used_countries_codes: &[Alpha3]) -> Vec<Country> {
    let available_countries = used_countries_codes
        .iter()
        .filter_map(|country_code| get_country(&countries_arg, country_code))
        .collect::<Vec<Country>>();

    let contains_all_countries = available_countries.iter().any(|country_| country_.parent == None);

    let mut result = vec![];
    if contains_all_countries {
        result.push(countries_arg.clone());
    } else {
        let mut countries_tree = countries_arg.clone();
        let used_codes: Vec<Alpha3> = available_countries.iter().map(|c| c.alpha3.clone()).collect();
        countries_tree = remove_unused_countries(countries_tree, &used_codes);

        result.push(countries_tree);
    }

    result
}

pub fn remove_unused_countries(mut country: Country, used_countries_codes: &[Alpha3]) -> Country {
    let mut children = vec![];
    for country_child in country.children {
        if used_countries_codes.iter().any(|used_code| country_child.alpha3 == *used_code) {
            children.push(country_child);
        } else {
            let new_country = remove_unused_countries(country_child, used_countries_codes);
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

    fn create_mock_countries_region1(parent_: Option<Alpha3>) -> Vec<Country> {
        vec![
            Country {
                label: "Russia".to_string().into(),
                children: vec![],
                level: 2,
                parent: parent_.clone(),
                alpha2: Alpha2("RU".to_string()),
                alpha3: Alpha3("RUS".to_string()),
                numeric: 0,
                is_selected: false,
            },
            Country {
                label: "Austria".to_string().into(),
                children: vec![],
                level: 2,
                parent: parent_.clone(),
                alpha2: Alpha2("AT".to_string()),
                alpha3: Alpha3("AUT".to_string()),
                numeric: 0,
                is_selected: false,
            },
        ]
    }

    fn create_mock_countries_region2(parent_: Option<Alpha3>) -> Vec<Country> {
        vec![Country {
            label: "Brazil".to_string().into(),
            children: vec![],
            level: 2,
            parent: parent_,
            alpha2: Alpha2("BR".to_string()),
            alpha3: Alpha3("BRA".to_string()),
            numeric: 0,
            is_selected: false,
        }]
    }

    fn create_mock_region3(parent_: Option<Alpha3>) -> Country {
        Country {
            label: "North America".to_string().into(),
            children: vec![],
            level: 2,
            parent: parent_,
            alpha2: Alpha2("".to_string()),
            alpha3: Alpha3("XNA".to_string()),
            numeric: 0,
            is_selected: false,
        }
    }

    fn create_mock_countries() -> (Country, Alpha3) {
        let root_code = Alpha3("XAL".to_string());

        let region1_alpha3 = Alpha3("XEU".to_string());
        let region_1 = Country {
            label: "Europe".to_string().into(),
            children: create_mock_countries_region1(Some(region1_alpha3.clone())),
            level: 1,
            parent: Some(root_code.clone()),
            alpha2: Alpha2("".to_string()),
            alpha3: region1_alpha3,
            numeric: 0,
            is_selected: false,
        };

        let region2_alpha3 = Alpha3("XSA".to_string());
        let region_2 = Country {
            label: "South America".to_string().into(),
            children: create_mock_countries_region2(Some(region2_alpha3.clone())),
            level: 1,
            parent: Some(root_code.clone()),
            alpha2: Alpha2("".to_string()),
            alpha3: region2_alpha3,
            numeric: 0,
            is_selected: false,
        };

        (
            Country {
                label: "All".to_string().into(),
                level: 0,
                parent: None,
                children: vec![region_1, region_2],
                alpha2: Alpha2("".to_string()),
                alpha3: root_code.clone(),
                numeric: 0,
                is_selected: false,
            },
            root_code,
        )
    }

    #[test]
    fn test_parent_countries() {
        let (country, _) = create_mock_countries();
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
        let (country, _) = create_mock_countries();
        let child_code = Alpha3("RUS".to_string());
        let new_country = get_country(&country, &child_code).unwrap();
        assert_eq!(new_country.alpha3, child_code.clone().into());
    }

    #[test]
    fn test_used_only_one_region() {
        let (mut country, _) = create_mock_countries();
        let region_alpha3 = Alpha3("XEU".to_string());
        let used_codes: Vec<Alpha3> = vec![region_alpha3.clone()];

        assert_eq!(country.children.len(), 2, "Mock countries not contains 2 regions");
        country = remove_unused_countries(country, &used_codes);
        assert_eq!(country.children.len(), 1);
        assert_eq!(country.children[0].alpha3, region_alpha3);
    }

    #[test]
    fn test_used_only_one_country_from_region() {
        let (mut country, _) = create_mock_countries();
        let region_code = Alpha3("XEU".to_string());
        let country_code = Alpha3("RUS".to_string());
        let used_codes = vec![country_code.clone()];

        {
            let region = country
                .children
                .iter()
                .find(|c| c.alpha3 == region_code)
                .expect(&format!("Not found region with code {:?} before run test", region_code));
            assert_eq!(region.children.len(), 2, "Mock countries not contains 2 countries");
        }

        country = remove_unused_countries(country, &used_codes);
        let region = country
            .children
            .iter()
            .find(|c| c.alpha3 == region_code)
            .expect(&format!("Not found region with code {:?} after test", region_code));
        assert_eq!(region.children.len(), 1);
        assert_eq!(region.children[0].alpha3, country_code);
    }

    #[test]
    fn test_used_country_from_region_plus_region() {
        let (mut country, root_code) = create_mock_countries();
        country.children.push(create_mock_region3(Some(root_code)));

        let country_code = Alpha3("RUS".to_string());
        let region_code2 = Alpha3("XSA".to_string());
        let used_codes = vec![country_code.clone(), region_code2.clone()];

        assert_eq!(country.children.len(), 3, "Mock countries not contains 3 regions before run test");
        country = remove_unused_countries(country, &used_codes);
        assert_eq!(country.children.len(), 2, "Mock countries not contains 2 regions after run test");
    }

}
