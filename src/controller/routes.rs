use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Roles,
    RoleById { id: RoleId },
    RolesByUserId { user_id: UserId },
    Restrictions,
    ShippingLocal,
    ShippingLocalById { base_product_id: BaseProductId },
    ShippingInternational,
    ShippingInternationalById { base_product_id: BaseProductId },
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut route_parser = RouteParser::default();

    route_parser.add_route(r"^/restrictions$", || Route::Restrictions);
    route_parser.add_route(r"^/shipping/local", || Route::ShippingLocal);
    route_parser.add_route_with_params(r"^/shipping/local/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|base_product_id| Route::ShippingLocalById { base_product_id })
    });
    route_parser.add_route(r"^/shipping/international", || Route::ShippingInternational);
    route_parser.add_route_with_params(r"^/shipping/international/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|base_product_id| Route::ShippingInternationalById { base_product_id })
    });
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

    route_parser
}
