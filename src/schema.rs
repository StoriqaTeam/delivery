table! {
    companies (id) {
        id -> Int4,
        name -> Varchar,
        label -> Varchar,
        description -> Nullable<Varchar>,
        deliveries_from -> Jsonb,
        logo -> Varchar,
        currency -> Varchar,
    }
}

table! {
    companies_packages (id) {
        id -> Int4,
        company_id -> Int4,
        package_id -> Int4,
        shipping_rate_source -> Varchar,
        dimensional_factor -> Nullable<Int4>,
    }
}

table! {
    countries (label) {
        label -> Varchar,
        level -> Int4,
        alpha2 -> Varchar,
        alpha3 -> Varchar,
        numeric -> Int4,
        parent -> Nullable<Varchar>,
    }
}

table! {
    packages (id) {
        id -> Int4,
        name -> Varchar,
        max_size -> Int4,
        min_size -> Int4,
        max_weight -> Int4,
        min_weight -> Int4,
        deliveries_to -> Jsonb,
    }
}

table! {
    pickups (id) {
        id -> Int4,
        base_product_id -> Int4,
        store_id -> Int4,
        pickup -> Bool,
        price -> Nullable<Float8>,
    }
}

table! {
    products (id) {
        id -> Int4,
        base_product_id -> Int4,
        store_id -> Int4,
        company_package_id -> Int4,
        price -> Nullable<Float8>,
        deliveries_to -> Jsonb,
        shipping -> Varchar,
        currency -> Varchar,
    }
}

table! {
    roles (id) {
        id -> Uuid,
        user_id -> Int4,
        name -> Varchar,
        data -> Nullable<Jsonb>,
    }
}

table! {
    shipping_rates (id) {
        id -> Int4,
        company_package_id -> Int4,
        from_alpha3 -> Varchar,
        to_alpha3 -> Varchar,
        rates -> Jsonb,
    }
}

table! {
    user_addresses (id) {
        id -> Int4,
        user_id -> Int4,
        administrative_area_level_1 -> Nullable<Varchar>,
        administrative_area_level_2 -> Nullable<Varchar>,
        country -> Varchar,
        locality -> Nullable<Varchar>,
        political -> Nullable<Varchar>,
        postal_code -> Varchar,
        route -> Nullable<Varchar>,
        street_number -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        is_priority -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        country_code -> Nullable<Varchar>,
    }
}

joinable!(companies_packages -> companies (company_id));
joinable!(companies_packages -> packages (package_id));
joinable!(products -> companies_packages (company_package_id));
joinable!(shipping_rates -> companies_packages (company_package_id));

allow_tables_to_appear_in_same_query!(
    companies,
    companies_packages,
    countries,
    packages,
    pickups,
    products,
    roles,
    shipping_rates,
    user_addresses,
);
