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
    restrictions (id) {
        id -> Int4,
        name -> Varchar,
        max_weight -> Float8,
        max_size -> Float8,
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
    delivery_from,
    delivery_to,
    restrictions,
    user_roles,
);
