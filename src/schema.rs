table! {
    companies (id) {
        id -> Int4,
        name -> Varchar,
        label -> Varchar,
        description -> Nullable<Varchar>,
        deliveries_from -> Jsonb,
        logo -> Varchar,
    }
}

table! {
    companies_packages (id) {
        id -> Int4,
        company_id -> Int4,
        package_id -> Int4,
    }
}

table! {
    countries (label) {
        label -> Varchar,
        name -> Jsonb,
        parent_label -> Nullable<Varchar>,
        level -> Int4,
    }
}

table! {
    delivery_from (id) {
        id -> Int4,
        company_id -> Varchar,
        country -> Varchar,
        restriction_name -> Varchar,
    }
}

table! {
    delivery_to (id) {
        id -> Int4,
        company_id -> Varchar,
        country -> Varchar,
        additional_info -> Nullable<Jsonb>,
    }
}

table! {
    international_shipping (id) {
        id -> Int4,
        base_product_id -> Int4,
        companies -> Jsonb,
        store_id -> Int4,
    }
}

table! {
    local_shipping (id) {
        id -> Int4,
        base_product_id -> Int4,
        pickup -> Bool,
        country -> Varchar,
        companies -> Jsonb,
        store_id -> Int4,
        pickup_price -> Nullable<Float8>,
    }
}

table! {
    packages (id) {
        id -> Int4,
        name -> Varchar,
        max_size -> Float8,
        min_size -> Float8,
        max_weight -> Float8,
        min_weight -> Float8,
        deliveries_to -> Jsonb,
    }
}

table! {
    pickups (id) {
        id -> Int4,
        base_product_id -> Int4,
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
    }
}

table! {
    restrictions (id) {
        id -> Int4,
        name -> Varchar,
        max_weight -> Float8,
        max_size -> Float8,
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

joinable!(companies_packages -> companies (company_id));
joinable!(companies_packages -> packages (package_id));
joinable!(products -> companies_packages (company_package_id));

allow_tables_to_appear_in_same_query!(
    companies,
    companies_packages,
    countries,
    delivery_from,
    delivery_to,
    international_shipping,
    local_shipping,
    packages,
    pickups,
    products,
    restrictions,
    roles,
);
