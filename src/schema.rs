table! {
    company_delivery_from (id) {
        id -> Int4,
        company_id -> Varchar,
        country -> Varchar,
        company_restriction -> Varchar,
    }
}

table! {
    company_delivery_to (id) {
        id -> Int4,
        company_id -> Varchar,
        country -> Varchar,
        additional_info -> Nullable<Jsonb>,
    }
}

table! {
    company_restrictions (id) {
        id -> Int4,
        name -> Varchar,
        max_weight -> Nullable<Float8>,
        max_size -> Nullable<Float8>,
    }
}

table! {
    user_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    company_delivery_from,
    company_delivery_to,
    company_restrictions,
    user_roles,
);
