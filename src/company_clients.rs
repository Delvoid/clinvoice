use diesel::RunQueryDsl;

use crate::db::establish_connection;
use crate::models::NewCompanyClient;

pub fn create_company_client(company_id: i32, client_id: i32) -> usize {
    use crate::schema::company_clients;

    let new_company_client = NewCompanyClient {
        company_id,
        client_id,
    };

    let conn = &mut establish_connection();

    diesel::insert_into(company_clients::table)
        .values(&new_company_client)
        .execute(conn)
        .expect("Error saving new company client")
}
