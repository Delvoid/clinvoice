use crate::schema::{clients, companies, company_clients, invoices, items};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = companies)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Company {
    pub id: i32,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Eq, Hash, Clone)]
#[diesel(table_name = clients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Queryable, Selectable, PartialEq, Debug)]
#[diesel(table_name = company_clients)]
#[diesel(belongs_to(Company))]
#[diesel(belongs_to(Client))]
#[diesel(primary_key(company_id, client_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CompanyClient {
    pub company_id: i32,
    pub client_id: i32,
}

#[derive(Queryable, Identifiable, Associations, Selectable, Debug, PartialEq)]
#[diesel(belongs_to(Client))]
#[diesel(belongs_to(Company))]
#[diesel(table_name = invoices)]
pub struct Invoice {
    pub id: i32,
    pub company_id: Option<i32>,
    pub company_name: String,
    pub company_address: Option<String>,
    pub company_email: Option<String>,
    pub company_phone: Option<String>,
    pub client_id: Option<i32>,
    pub client_name: String,
    pub client_address: Option<String>,
    pub client_email: Option<String>,
    pub client_phone: Option<String>,
    pub date: String,
    pub total_amount: f64,
    pub logo_url: Option<String>,
    pub tax: Option<f64>,
    pub notes: Option<String>,
    pub regenerated: Option<bool>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = items)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]

pub struct Item {
    pub id: i32,
    pub invoice_id: i32,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total: f64,
}

#[derive(Insertable)]
#[diesel(table_name = companies)]
pub struct NewCompany<'a> {
    pub name: &'a str,
    pub address: Option<&'a str>,
    pub email: Option<&'a str>,
    pub phone: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = clients)]
pub struct NewClient<'a> {
    pub name: &'a str,
    pub address: Option<&'a str>,
    pub email: Option<&'a str>,
    pub phone: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = company_clients)]
pub struct NewCompanyClient {
    pub company_id: i32,
    pub client_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = invoices)]
pub struct NewInvoice {
    pub company_id: Option<i32>,
    pub company_name: String,
    pub company_address: Option<String>,
    pub company_email: Option<String>,
    pub company_phone: Option<String>,
    pub client_id: Option<i32>,
    pub client_name: String,
    pub client_address: Option<String>,
    pub client_email: Option<String>,
    pub client_phone: Option<String>,
    pub date: String,
    pub total_amount: f64,
    pub tax: Option<f64>,
    pub notes: Option<String>,
    pub regenerated: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = items)]
pub struct NewItem {
    pub invoice_id: i32,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total: f64,
}
