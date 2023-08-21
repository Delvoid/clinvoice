use crate::db::establish_connection;
use crate::models::{Invoice, Item, NewInvoice, NewItem};
use crate::pdf::CompanyPdf;
use crate::schema::{invoices, items};
use crate::utils::get_input;
use crate::{pdf, InvoiceGenerateArgs};
use cli_table::{print_stdout, Cell, Style, Table};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use text_colorizer::*;

#[derive(Clone, Debug)]
pub struct InvoiceArgs {
    pub company: Option<String>,
    pub client: Option<String>,
    pub item: Option<Vec<String>>,
    pub notes: Option<String>,
    pub custom: Option<bool>,
}

fn collect_items(used_args: bool) -> Vec<String> {
    if used_args {
        println!("{}", "Items cannot be empty".red());
    }
    let mut items = Vec::new();

    'item_loop: loop {
        let item_name = loop {
            let item_name = get_input(&format!(
                "Enter item name: {}:",
                "(or 'done' to finish)".bright_black()
            ));
            if item_name == "done" {
                break 'item_loop;
            }

            if item_name.trim().is_empty() {
                println!("{}", "Item name cannot be empty".red());
                continue;
            }

            if !item_name.trim().is_empty() {
                break item_name;
            }
        };

        let item_price = loop {
            let price = get_input("Enter item price: ");

            match price.parse::<f32>() {
                Ok(price) => break price,
                Err(_) => {
                    println!("{}", "Please enter a valid price".red());
                    continue;
                }
            }
        };

        let item_quantity = loop {
            let quantity = get_input("Enter item quantity: ");

            match quantity.parse::<u32>() {
                Ok(quantity) => break quantity,
                Err(_) => {
                    println!("{}", "Please enter a valid quantity".red());
                    continue;
                }
            }
        };

        let item = format!(
            "{{\"description\": \"{}\",\"quantity\": {}, \"price\": {}}}",
            item_name, item_quantity, item_price
        );
        items.push(item);
    }

    items
}

pub fn generate_invoice(
    mut args: InvoiceGenerateArgs,
    used_args: bool,
    custom: bool,
) -> Result<(), String> {
    if args.company_name.is_none() && args.client_name.is_none() {
        let company_name = get_input(&format!(
            "Enter the company name: {}:",
            "(optional)".bright_black()
        ));

        if !company_name.trim().is_empty() {
            args.company_name = Some(company_name);
        }

        if args.company_name.is_some() && !used_args {
            let company_address = get_input(&format!(
                "Enter the company address: {}:",
                "(optional)".bright_black()
            ));

            if !company_address.trim().is_empty() {
                args.company_address = Some(company_address);
            }

            let company_email = get_input(&format!(
                "Enter the company email: {}:",
                "(optional)".bright_black()
            ));

            if !company_email.trim().is_empty() {
                args.company_email = Some(company_email);
            }

            let company_number = get_input(&format!(
                "Enter the company number: {}:",
                "(optional)".bright_black()
            ));

            if !company_number.trim().is_empty() {
                args.company_number = Some(company_number);
            }
        }
    }
    args.client_name.get_or_insert_with(|| loop {
        if used_args {
            println!("{}", "Client name cannot be empty".red());
        }
        let client_name = get_input("Enter the client name: ");

        if client_name.trim().is_empty() {
            println!("{}", "Client name cannot be empty".red());
            continue;
        }

        if args.client_address.is_none() && !used_args && custom {
            let client_address = get_input(&format!(
                "Enter the client address: {}:",
                "(optional)".bright_black()
            ));

            if !client_address.trim().is_empty() {
                args.client_address = Some(client_address);
            }
        }

        if args.client_email.is_none() && !used_args && custom {
            let client_email = get_input(&format!(
                "Enter the client email: {}:",
                "(optional)".bright_black()
            ));

            if !client_email.trim().is_empty() {
                args.client_email = Some(client_email);
            }
        }

        if !client_name.trim().is_empty() {
            break client_name;
        }

        if args.client_number.is_none() && !used_args && custom {
            let client_number = get_input(&format!(
                "Enter the client number: {}:",
                "(optional)".bright_black()
            ));

            if !client_number.trim().is_empty() {
                args.client_number = Some(client_number);
            }
        }
    });

    args.item.get_or_insert_with(|| collect_items(used_args));

    if args.notes.is_none() && !used_args {
        let notes = get_input(&format!(
            "Enter notes for the invoice: {}:",
            "(optional)".bright_black()
        ));

        if !notes.trim().is_empty() {
            args.notes = Some(notes);
        }
    }

    let items_result: Result<Vec<pdf::Item>, _> = args
        .item
        .unwrap()
        .iter()
        .map(|item| {
            serde_json::from_str(item).map_err(|e| {
                format!(
                    "{} parsing item: {}. Check your input and try again. \n{}",
                    "Error".red(),
                    item.blue(),
                    format!("Error: {}", e).red()
                )
            })
        })
        .collect();

    match items_result {
        Ok(items) => {
            let company = args.company_name;

            let company = CompanyPdf {
                name: company,
                address: args.company_address,
                email: args.company_email,
                phone: args.company_number,
            };
            let client = args
                .client_name
                .expect("Client name should have been initialized by now");

            let client = pdf::ClientPdf {
                name: client,
                address: args.client_address,
                email: args.client_email,
                phone: args.client_number,
            };

            let custom = args.custom.unwrap_or_default();

            match pdf::generate_pdf(company, client, items, args.notes, custom, false) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("Error: {}", e);
                    Err(e.to_string())
                }
            }
        }
        Err(e) => {
            println!("An error occurred: {}", e);
            Err(e)
        }
    }
}

pub fn create_invoice_with_items(new_invoice: &NewInvoice, items: Vec<NewItem>) {
    let conn = &mut establish_connection();

    diesel::insert_into(invoices::table)
        .values(new_invoice)
        .execute(conn)
        .expect("Error saving new client");

    let invoice_id = latest_id();

    for mut item in items {
        item.invoice_id = invoice_id;
        diesel::insert_into(items::table)
            .values(&item)
            .execute(conn)
            .expect("Error saving new item");
    }
}

pub fn latest_id() -> i32 {
    let conn = &mut establish_connection();

    match invoices::table
        .order(invoices::id.desc())
        .select(invoices::id)
        .first::<i32>(conn)
    {
        Ok(id) => id,
        Err(diesel::result::Error::NotFound) => 0,
        Err(e) => panic!("Error retrieving latest ID: {:?}", e),
    }
}

pub enum InvoiceSearchField {
    Id,
    Client,
}

pub fn search_by(field: InvoiceSearchField, search_value: &str, print_table: Option<bool>) {
    let conn = &mut establish_connection();

    let mut query = invoices::table.into_boxed();

    query = match field {
        InvoiceSearchField::Id => {
            query.filter(invoices::id.eq(search_value.parse::<i32>().unwrap_or(-1)))
        }
        InvoiceSearchField::Client => query.filter(invoices::client_name.eq(search_value)),
    };

    let invoice_results = query
        .select(invoices::all_columns)
        .load(conn)
        .expect("Error loading clients with companies");

    if invoice_results.is_empty() {
        let query_type = match field {
            InvoiceSearchField::Id => "id".red(),
            InvoiceSearchField::Client => "client".red(),
        };
        println!(
            "{} {}{} {}",
            "No invoice found using".red(),
            query_type,
            ":".red(),
            search_value
        );
        return;
    }

    if print_table.unwrap_or(true) {
        invoice_table(&invoice_results);

        if invoice_results.len() == 1 {
            // After printing the invoice, we need to fetch and print its items
            let invoice_id = invoice_results.first().map(|i| i.id).unwrap_or_default();
            let item_results = items::table
                .filter(items::invoice_id.eq(invoice_id))
                .load::<Item>(conn)
                .expect("Error loading items for the invoice");

            items_table(&item_results);
        }
    }
}

pub fn regenerate(id: i32) -> Result<(), Box<dyn std::error::Error>> {
    let conn = &mut establish_connection();

    let invoice = invoices::table
        .find(id)
        .first::<Invoice>(conn)
        .expect("Error loading invoice");

    let items = items::table
        .filter(items::invoice_id.eq(id))
        .load::<Item>(conn)
        .expect("Error loading items for the invoice");

    // format items std::vec::Vec<pfd::Item>
    let items = items
        .iter()
        .map(|item| pdf::Item {
            description: item.description.clone(),
            quantity: item.quantity as u32,
            price: item.unit_price as f64,
        })
        .collect::<Vec<_>>();

    let company = CompanyPdf {
        name: Some(invoice.company_name),
        address: invoice.company_address,
        email: invoice.company_email,
        phone: invoice.company_phone,
    };
    let client = pdf::ClientPdf {
        name: invoice.client_name,
        address: invoice.client_address,
        email: invoice.client_email,
        phone: invoice.client_phone,
    };

    match pdf::generate_pdf(company, client, items, invoice.notes, true, true) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {}", e);

            Err(format!("{}", e).into())
        }
    }
}

fn invoice_table(invoices: &[Invoice]) {
    let invoices = invoices
        .iter()
        .map(|invoice| {
            vec![
                invoice.id.to_string().cell(),
                invoice.client_name.clone().cell(),
                invoice.company_name.clone().cell(),
                invoice.date.clone().cell(),
                invoice.total_amount.to_string().cell(),
                invoice.notes.clone().unwrap_or_default().cell(),
            ]
        })
        .collect::<Vec<_>>();

    let table = invoices
        .table()
        .title(vec![
            "ID".cell(),
            "Client Name".cell(),
            "Company Name".cell(),
            "Date".cell(),
            "Total Amount".cell(),
            "Notes".cell(),
        ])
        .bold(true);

    // Print the table
    print_stdout(table).expect("Error printing the table");
}

fn items_table(items: &[Item]) {
    let items = items
        .iter()
        .map(|item| {
            vec![
                item.id.to_string().cell(),
                item.description.clone().cell(),
                item.quantity.to_string().cell(),
                item.unit_price.to_string().cell(),
                format!("{:.2}", item.total).cell(),
            ]
        })
        .collect::<Vec<_>>();

    let table = items
        .table()
        .title(vec![
            "ID".cell(),
            "Description".cell(),
            "Quantity".cell(),
            "Unit Price".cell(),
            "Total".cell(),
        ])
        .bold(true);

    println!("+-------------------------+");
    println!("|--------- {} ---------|", "Items".bold());
    println!("+-------------------------+");

    // Print the table
    print_stdout(table).expect("Error printing the table");
}

pub fn list_invoices() -> Vec<Invoice> {
    use crate::schema::invoices::dsl::*;

    let conn = &mut establish_connection();

    let list = invoices
        .load::<Invoice>(conn)
        .expect("Error loading invoices");

    invoice_table(&list);

    list
}
