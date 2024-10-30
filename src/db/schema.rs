// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Text,
        title -> Varchar,
        body -> Text,
        published -> Bool,
        user_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        name -> Varchar,
        email -> Text,
        password -> Nullable<Text>,
        verified -> Bool,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
