use crate::db::establish_connection;
use crate::models::{Company, NewCompany};
use crate::schema::companies;
use crate::utils::get_input;
use diesel::prelude::*;
use text_colorizer::*;

pub enum CompanySearchField {
    Name,
    Address,
}

impl Company {
    pub fn create(
        name: &str,
        address: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Company {
        let new_company = NewCompany {
            name,
            address,
            email,
            phone,
        };

        let conn = &mut establish_connection();

        diesel::insert_into(companies::table)
            .values(&new_company)
            .returning(Company::as_returning())
            .get_result(conn)
            .expect("Error saving new client")
    }
    pub fn default_company() -> Company {
        use crate::schema::companies::dsl::*;

        let default_company_id = crate::config::load_config().default_company;

        let conn = &mut establish_connection();

        companies
            .filter(id.eq(default_company_id))
            .first::<Company>(conn)
            .expect("Error loading company")
    }

    pub fn search_by(field: CompanySearchField, search_value: &str) -> Vec<Company> {
        let conn = &mut establish_connection();

        let mut query = companies::table.into_boxed();

        query = match field {
            CompanySearchField::Name => {
                query.filter(companies::name.like(format!("%{}%", search_value)))
            }
            CompanySearchField::Address => {
                query.filter(companies::address.like(format!("%{}%", search_value)))
            }
        };

        query
            .select(companies::all_columns)
            .load::<Company>(conn)
            .expect("Error loading clients with companies")
    }

    pub fn select_company(companies: &Vec<Company>) -> Option<&Company> {
        // Display the clients and let the user select one
        println!(
            "{}",
            "Multiple companies found. \nSelect a company:".green()
        );
        let mut i = 1;
        for company in companies {
            let company_address = company.address.clone().unwrap_or_default();

            println!("{}: {} - {}", i, company.name, company_address);
            i += 1;
        }

        // Get the user's selection
        let mut selection_str = String::new();
        std::io::stdin()
            .read_line(&mut selection_str)
            .expect("Failed to read line");
        let selection = selection_str.trim().parse::<usize>().unwrap();

        // Get the selected client
        companies.get(selection - 1)
    }
}

pub fn create_company_prompts(
    name: Option<String>,
    address: Option<String>,
    email: Option<String>,
    phone: Option<String>,
) -> Company {
    let name = match name {
        Some(name) => name,
        None => get_input("Enter company name: "),
    };

    let address = match address {
        Some(address) => Some(address),
        None => {
            let address_input = get_input(&format!(
                "Enter company address {}:",
                "(optional)".bright_black()
            ));
            if address_input.is_empty() {
                None
            } else {
                Some(address_input)
            }
        }
    };

    let email = match email {
        Some(email) => Some(email),
        None => {
            let email_input = get_input(&format!(
                "Enter company email {}:",
                "(optional)".bright_black()
            ));
            if email_input.is_empty() {
                None
            } else {
                Some(email_input)
            }
        }
    };

    let phone = match phone {
        Some(phone) => Some(phone),
        None => {
            let phone_input = get_input(&format!(
                "Enter company phone {}:",
                "(optional)".bright_black()
            ));
            if phone_input.is_empty() {
                None
            } else {
                Some(phone_input)
            }
        }
    };

    let company = Company::create(
        &name,
        address.as_deref(),
        email.as_deref(),
        phone.as_deref(),
    );

    println!("{} ", format!("Company {} added", company.name).green());

    company
}
