use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Roles,
    RoleById {
        id: RoleId,
    },
    RolesByUserId {
        user_id: UserId,
    },
    Countries,
    CountriesFlatten,
    CountryByAlpha2 {
        alpha2: Alpha2,
    },
    CountryByAlpha3 {
        alpha3: Alpha3,
    },
    CountryByNumeric {
        numeric: i32,
    },
    Products,
    ProductsById {
        base_product_id: BaseProductId,
    },
    ProductsByIdAndCompanyPackageId {
        base_product_id: BaseProductId,
        company_package_id: CompanyPackageId,
    },
    Companies,
    CompanyById {
        company_id: CompanyId,
    },
    Packages,
    PackagesById {
        package_id: PackageId,
    },
    CompaniesPackages,
    CompaniesPackagesById {
        company_package_id: CompanyPackageId,
    },
    PackagesByCompanyId {
        company_id: CompanyId,
    },
    CompaniesByPackageId {
        package_id: PackageId,
    },
    AvailablePackages,
    AvailablePackagesForUser {
        base_product_id: BaseProductId,
    },
    UsersAddresses,
    UserAddress {
        user_id: UserId,
    },
    UserAddressById {
        user_address_id: i32,
    },
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut route_parser = RouteParser::default();

    route_parser.add_route(r"^/roles$", || Route::Roles);
    route_parser.add_route_with_params(r"^/roles/by-user-id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_id| Route::RolesByUserId { user_id })
    });
    route_parser.add_route_with_params(r"^/roles/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|id| Route::RoleById { id })
    });

    route_parser.add_route(r"^/countries$", || Route::Countries);
    route_parser.add_route(r"^/countries/flatten$", || Route::CountriesFlatten);

    // Countries search
    route_parser.add_route_with_params(r"^/countries/alpha2/(\S+)$", |params| {
        params
            .get(0)
            .map(|param| param.to_string().to_uppercase())
            .map(Alpha2)
            .map(|alpha2| Route::CountryByAlpha2 { alpha2 })
    });

    route_parser.add_route_with_params(r"^/countries/alpha3/(\S+)$", |params| {
        params
            .get(0)
            .map(|param| param.to_string().to_uppercase())
            .map(Alpha3)
            .map(|alpha3| Route::CountryByAlpha3 { alpha3 })
    });

    route_parser.add_route_with_params(r"^/countries/numeric/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|numeric| Route::CountryByNumeric { numeric })
    });

    route_parser.add_route(r"^/products$", || Route::Products);
    route_parser.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|base_product_id| Route::ProductsById { base_product_id })
    });
    route_parser.add_route_with_params(r"^/products/(\d+)/company_package/(\d+)$", |params| {
        if let Some(base_product_id_s) = params.get(0) {
            if let Some(company_package_id_s) = params.get(1) {
                if let Ok(base_product_id) = base_product_id_s.parse().map(BaseProductId) {
                    if let Ok(company_package_id) = company_package_id_s.parse().map(CompanyPackageId) {
                        return Some(Route::ProductsByIdAndCompanyPackageId {
                            base_product_id,
                            company_package_id,
                        });
                    }
                }
            }
        }
        None
    });

    route_parser.add_route(r"^/companies$", || Route::Companies);
    route_parser.add_route_with_params(r"^/companies/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|company_id| Route::CompanyById { company_id })
    });

    route_parser.add_route(r"^/packages$", || Route::Packages);
    route_parser.add_route_with_params(r"^/packages/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|package_id| Route::PackagesById { package_id })
    });

    route_parser.add_route(r"^/companies_packages$", || Route::CompaniesPackages);
    route_parser.add_route_with_params(r"^/companies_packages/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|company_package_id| Route::CompaniesPackagesById { company_package_id })
    });

    route_parser.add_route_with_params(r"^/companies/(\d+)/packages$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|company_id| Route::PackagesByCompanyId { company_id })
    });

    route_parser.add_route_with_params(r"^/packages/(\d+)/companies$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|package_id| Route::CompaniesByPackageId { package_id })
    });
    route_parser.add_route(r"^/available_packages$", || Route::AvailablePackages);
    route_parser.add_route_with_params(r"^/available_packages_for_user/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|base_product_id| Route::AvailablePackagesForUser { base_product_id })
    });

    // /users/addresses route
    route_parser.add_route(r"^/users/addresses$", || Route::UsersAddresses);

    // /users/:id/addresses route
    route_parser.add_route_with_params(r"^/users/(\d+)/addresses$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_id| Route::UserAddress { user_id })
    });

    // /users/addresses/:id route
    route_parser.add_route_with_params(r"^/users/addresses/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_address_id| Route::UserAddressById { user_address_id })
    });

    route_parser
}
