// @generated automatically by Diesel CLI.

diesel::table! {
    clients (id) {
        id -> Integer,
        name -> Text,
        address -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
    }
}

diesel::table! {
    companies (id) {
        id -> Integer,
        name -> Text,
        address -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
    }
}

diesel::table! {
    company_clients (company_id, client_id) {
        company_id -> Integer,
        client_id -> Integer,
    }
}

diesel::table! {
    invoices (id) {
        id -> Integer,
        company_id -> Nullable<Integer>,
        company_name -> Text,
        company_address -> Nullable<Text>,
        company_email -> Nullable<Text>,
        company_phone -> Nullable<Text>,
        client_id -> Nullable<Integer>,
        client_name -> Text,
        client_address -> Nullable<Text>,
        client_email -> Nullable<Text>,
        client_phone -> Nullable<Text>,
        date -> Text,
        total_amount -> Double,
        logo_url -> Nullable<Text>,
        tax -> Nullable<Double>,
        notes -> Nullable<Text>,
        regenerated -> Nullable<Bool>,
    }
}

diesel::table! {
    items (id) {
        id -> Integer,
        invoice_id -> Integer,
        description -> Text,
        quantity -> Integer,
        unit_price -> Double,
        total -> Double,
    }
}

diesel::joinable!(company_clients -> clients (client_id));
diesel::joinable!(company_clients -> companies (company_id));
diesel::joinable!(invoices -> clients (client_id));
diesel::joinable!(invoices -> companies (company_id));
diesel::joinable!(items -> invoices (invoice_id));

diesel::allow_tables_to_appear_in_same_query!(
    clients,
    companies,
    company_clients,
    invoices,
    items,
);
