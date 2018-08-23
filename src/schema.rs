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
    roles (id) {
        id -> Uuid,
        user_id -> Int4,
        name -> Varchar,
        data -> Nullable<Jsonb>,
    }
}

allow_tables_to_appear_in_same_query!(
    delivery_from,
    delivery_to,
    restrictions,
    roles,
);
