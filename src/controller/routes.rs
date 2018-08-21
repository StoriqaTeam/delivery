use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    UserRoles,
    UserRole(UserId),
    DefaultRole(UserId),
    Restrictions,
    Restriction(String),
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut router = RouteParser::default();

    // User_roles Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // User_roles/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(UserId)
            .map(Route::UserRole)
    });

    // roles/default/:id route
    router.add_route_with_params(r"^/roles/default/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(UserId)
            .map(Route::DefaultRole)
    });

    // restrictions/:name route
    router.add_route_with_params(r"^/restrictions/by-name/([a-zA-Z0-9-_]+)$", |params| {
        params
            .get(0)
            .and_then(|restriction_name| restriction_name.parse().ok())
            .map(Route::Restriction)
    });

    router.add_route(r"^/restrictions$", || Route::Restrictions);

    router
}
