use crate::db::establish_connection;
use crate::models::{Client, Company, NewClient};
use crate::schema::{clients, companies, company_clients};
use crate::utils::get_input;
use cli_table::{print_stdout, Cell, Style, Table};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use indexmap::IndexMap;
use text_colorizer::*;

pub enum ClientSearchField {
    Name,
    Address,
    Company,
}

// Helper function to process query results
fn process_results(
    results: Vec<(Client, Company)>,
    print_table: bool,
) -> IndexMap<Client, Vec<String>> {
    let mut map = IndexMap::new();
    for (client, company) in results {
        let companies = map.entry(client).or_insert(Vec::new());
        companies.push(company.name);
    }

    if print_table {
        client_table(&map);
    }

    map
}

impl Client {
    pub fn create(
        name: &str,
        address: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Client {
        let new_client = NewClient {
            name,
            address,
            email,
            phone,
        };

        let conn = &mut establish_connection();

        diesel::insert_into(clients::table)
            .values(&new_client)
            .returning(Client::as_returning())
            .get_result(conn)
            .expect("Error saving new client")
    }

    fn with_companies() -> IndexMap<Client, Vec<String>> {
        let conn = &mut establish_connection();

        let results = clients::table
            .inner_join(company_clients::table.on(clients::id.eq(company_clients::client_id)))
            .inner_join(companies::table.on(companies::id.eq(company_clients::company_id)))
            .select((clients::all_columns, companies::all_columns))
            .load::<(Client, Company)>(conn)
            .expect("Error loading clients with companies");

        process_results(results, true)
    }
    pub fn search_by(
        field: ClientSearchField,
        search_value: &str,
        print_table: Option<bool>,
    ) -> IndexMap<Client, Vec<String>> {
        let conn = &mut establish_connection();

        let mut query = clients::table
            .inner_join(company_clients::table.on(clients::id.eq(company_clients::client_id)))
            .inner_join(companies::table.on(companies::id.eq(company_clients::company_id)))
            .into_boxed();

        query = match field {
            ClientSearchField::Name => {
                query.filter(clients::name.like(format!("%{}%", search_value)))
            }
            ClientSearchField::Address => {
                query.filter(clients::address.like(format!("%{}%", search_value)))
            }
            ClientSearchField::Company => {
                query.filter(companies::name.like(format!("%{}%", search_value)))
            }
        };

        let results = query
            .select((clients::all_columns, companies::all_columns))
            .order(clients::name.asc())
            .load::<(Client, Company)>(conn)
            .expect("Error loading clients with companies");

        let should_print = print_table.unwrap_or(true);

        process_results(results, should_print)
    }

    pub fn find_by_id(user_id: i32) -> Option<Client> {
        let conn = &mut establish_connection();

        clients::table
            .find(user_id)
            .first::<Client>(conn)
            .optional()
            .expect("Error loading client")
    }

    pub fn select_client(clients: &IndexMap<Client, Vec<String>>) -> Option<&Client> {
        // Display the clients and let the user select one
        println!("{}", "Multiple clients found. \nSelect a client:".green());
        let mut i = 1;
        for client in clients.keys() {
            let client_address = client.address.clone().unwrap_or_default();

            println!("{}: {} - {}", i, client.name, client_address);
            i += 1;
        }

        // Get the user's selection
        let mut selection_str = String::new();
        std::io::stdin()
            .read_line(&mut selection_str)
            .expect("Failed to read line");
        let selection = selection_str.trim().parse::<usize>().unwrap();

        // Get the selected client
        clients.iter().nth(selection - 1).map(|(client, _)| client)
    }
}

pub fn search_by_name(
    search_name: &str,
    print_table: Option<bool>,
) -> IndexMap<Client, Vec<String>> {
    Client::search_by(ClientSearchField::Name, search_name, print_table)
}
pub fn search_by_address(
    search_address: &str,
    print_table: Option<bool>,
) -> IndexMap<Client, Vec<String>> {
    Client::search_by(ClientSearchField::Address, search_address, print_table)
}

pub fn search_by_company(
    search_company: &str,
    print_table: Option<bool>,
) -> IndexMap<Client, Vec<String>> {
    Client::search_by(ClientSearchField::Company, search_company, print_table)
}

pub fn create_client_prompts(
    name: Option<String>,
    address: Option<String>,
    email: Option<String>,
    phone: Option<String>,
) -> Client {
    let name = match name {
        Some(name) => name,
        None => get_input("Enter client name: "),
    };

    let address = match address {
        Some(address) => Some(address),
        None => {
            let address_input = get_input(&format!(
                "Enter client address {}:",
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
                "Enter client email {}:",
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
                "Enter client phone {}:",
                "(optional)".bright_black()
            ));
            if phone_input.is_empty() {
                None
            } else {
                Some(phone_input)
            }
        }
    };

    let client = Client::create(
        &name,
        address.as_deref(),
        email.as_deref(),
        phone.as_deref(),
    );

    let default_company_id = crate::config::load_config().default_company;

    // create company client many-to-many relationship
    crate::company_clients::create_company_client(default_company_id, client.id);

    println!("{} ", format!("Client {} added", client.name).green());

    client
}

pub fn list_clients() -> Vec<Client> {
    use crate::schema::clients::dsl::*;

    let conn = &mut establish_connection();

    let list = clients.load::<Client>(conn).expect("Error loading clients");

    Client::with_companies();

    list
}

fn client_table(clients: &IndexMap<Client, Vec<String>>) {
    let clients = clients
        .iter()
        .map(|(client, company_names)| {
            let company_names = company_names.join(", ");
            vec![
                client.id.to_string().cell(),
                client.name.clone().cell(),
                client.address.clone().unwrap_or_default().cell(),
                company_names.cell(),
                client.email.clone().unwrap_or_default().cell(),
                client.phone.clone().unwrap_or_default().cell(),
            ]
        })
        .collect::<Vec<_>>();

    let table = clients
        .table()
        .title(vec![
            "ID".cell(),
            "Name".cell(),
            "Address".cell(),
            "Companies".cell(),
            "Email".cell(),
            "Phone".cell(),
        ])
        .bold(true);

    // Print the table
    print_stdout(table).expect("Error printing the table");
}
