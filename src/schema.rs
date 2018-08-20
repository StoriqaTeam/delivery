table! {
    user_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
