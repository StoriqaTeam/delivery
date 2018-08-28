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

allow_tables_to_appear_in_same_query!(
    companies,
    delivery_from,
    delivery_to,
    international_shipping,
    local_shipping,
    restrictions,
    roles,
);
