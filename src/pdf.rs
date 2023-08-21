use crate::config;
use crate::invoice::{self, create_invoice_with_items};
use crate::models::{Client, Company, NewInvoice};
use base64::{engine::general_purpose, Engine as _};
use handlebars::Handlebars;
use handlebars::{Context, Helper, HelperDef, HelperResult, Output, RenderContext};
use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptionsBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use text_colorizer::*;
use urlencoding;

struct AddressHelper;

impl HelperDef for AddressHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &Handlebars<'reg>,
        _: &Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(param) = h.param(0) {
            let address = param.value().as_str().unwrap_or("");
            let separated = Vec::from_iter(address.split(',').map(|s| s.trim()));
            let formatted = separated.join("<br/>");
            out.write(&formatted)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub description: String,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Default)]
pub struct CompanyPdf {
    pub name: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Default)]
pub struct ClientPdf {
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

fn get_total_amount(items: &[Item]) -> f64 {
    let total_amount = items
        .iter()
        .fold(0.0, |acc, item| acc + item.price * item.quantity as f64);
    (total_amount * 100.0).round() / 100.0
}

fn get_image_data_url() -> Result<String, Box<dyn std::error::Error>> {
    let config = config::load_config();
    // Read the image file into a byte vector
    let image_data = fs::read(config.logo_path)?;
    // Base64 encode the image data
    let image_data_base64 = general_purpose::STANDARD.encode(image_data);
    // Convert to a Data URL
    Ok(format!("data:image/png;base64,{}", image_data_base64))
}

fn process_template(
    template_string: String,
    data: serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("format_address", Box::new(AddressHelper));
    handlebars.register_template_string("template", template_string)?;
    let rendered = handlebars.render("template", &data)?;
    Ok(rendered)
}

fn write_pdf(rendered: String, invoice_number: String) -> Result<(), Box<dyn std::error::Error>> {
    let options = LaunchOptionsBuilder::default().build()?;
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&format!(
        "data:text/html,{}",
        urlencoding::encode(&rendered)
    ))?;
    while tab.find_element("body").is_err() {
        thread::sleep(Duration::from_millis(200)); // check every 100 milliseconds
    }

    let pdf_options: Option<PrintToPdfOptions> = Some(PrintToPdfOptions {
        print_background: Some(true),
        ..Default::default()
    });

    let pdf_data = tab.print_to_pdf(pdf_options)?;

    let today = chrono::offset::Local::now();

    let year = today.format("%Y");
    let month = today.format("%b");

    let config = config::load_config();

    // Construct path
    let path_str = format!(
        "{}/{}/{}/{}.pdf",
        config.invoice_path, year, month, invoice_number
    );

    let path = Path::new(&path_str);

    // Create directories if they don't exist
    if let Err(e) = fs::create_dir_all(path.parent().unwrap()) {
        println!("Failed to create directories: {}", e);
    }

    let mut file = File::create(&path_str)?;
    file.write_all(&pdf_data)?;

    println!("Invoice PDF saved to: {}", &path_str.bright_black());

    Ok(())
}

fn get_company(company_name: Option<String>) -> Result<Company, Box<dyn std::error::Error>> {
    let company = match company_name {
        Some(company_name) => crate::models::Company::search_by(
            crate::company::CompanySearchField::Name,
            &company_name,
        ),
        None => vec![crate::models::Company::default_company()],
    };

    if company.is_empty() {
        return Err(format!("{}", "Company not found".red()).into());
    }

    let company = if company.len() > 1 {
        crate::models::Company::select_company(&company)
    } else {
        company.first()
    };

    match company {
        Some(company) => Ok(company.clone()),
        None => Err(format!("{}", "No company selected".red()).into()),
    }
}

fn get_client(client_name: String) -> Result<Client, Box<dyn std::error::Error>> {
    let clients = crate::client::search_by_name(&client_name, Some(false));

    if clients.is_empty() {
        return Err(format!("{}", "Client not found".red()).into());
    }

    let client = if clients.len() > 1 {
        Client::select_client(&clients)
    } else {
        clients.iter().next().map(|(client, _)| client)
    };

    match client {
        Some(client) => Ok(client.clone()),
        None => Err(format!("{}", "No client selected".red()).into()),
    }
}

pub fn generate_pdf(
    company: CompanyPdf,
    client: ClientPdf,
    items: Vec<Item>,
    notes: Option<String>,
    custom: bool,
    regen: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // let mut file = File::open("template.html")?;
    // let mut template_string = String::new();
    // file.read_to_string(&mut template_string)?;

    let template_string = include_str!("template.html").to_string();

    let total_amount = get_total_amount(&items);
    let image_data_url = get_image_data_url()?;

    //todays day Month Year
    let today = chrono::offset::Local::now();

    let invoice_number = invoice::latest_id() + 1;
    let formatted_invoice_number = format!("{:05}", invoice_number);

    let data = if custom {
        let company_name = company
            .name
            .unwrap_or(crate::models::Company::default_company().name);

        json!({
            "invoice_number": formatted_invoice_number,
            "created_date": today.format("%d %B %Y").to_string(),
            "client_name": client.name,
            "client_address": client.address.unwrap_or_default(),
            "client_email": client.email.unwrap_or_default(),
            "client_phone": client.phone.unwrap_or_default(),
            "company_name": company_name,
            "company_address": company.address.unwrap_or_default(),
            "company_email": company.email.unwrap_or_default(),
            "company_phone": company.phone.unwrap_or_default(),
            "items": items,
            "total": total_amount,
            "logo_url": image_data_url,
            "tax": "0.00",
            "notes": notes.unwrap_or_default(),
        })
    } else {
        let company = get_company(company.name)?;
        let client = get_client(client.name)?;

        let company_name = company.name;
        let company_address = company.address.unwrap_or_default();

        let client_name = client.name;
        let client_address = client.address.unwrap_or_default();

        json!({
            "invoice_number": formatted_invoice_number,
            "created_date": today.format("%d %B %Y").to_string(),
            "client_name": client_name,
            "client_address": client_address,
            "company_name": company_name,
            "company_address": company_address,
            "items": items,
            "total": total_amount,
            "logo_url": image_data_url,
            "tax": "0.00",
            "notes": notes.unwrap_or_default(),
        })
    };

    print!("{}", "Generating invoice... \n".yellow());

    let rendered = process_template(template_string, data.clone())?;

    let pdf = write_pdf(rendered, formatted_invoice_number);

    match pdf {
        Ok(_) => {
            create_invoice_with_items(
                &NewInvoice {
                    company_id: Some(1), // This should be the actual company_id
                    company_name: data["company_name"]
                        .as_str()
                        .map_or(String::new(), ToString::to_string),
                    company_address: data["company_address"].as_str().map(|s| s.to_string()),
                    company_email: data["company_email"].as_str().map(|s| s.to_string()),
                    company_phone: data["company_phone"].as_str().map(|s| s.to_string()),
                    client_id: Some(1), // This should be the actual client_id
                    client_name: data["client_name"]
                        .as_str()
                        .map_or(String::new(), ToString::to_string),
                    client_address: data["client_address"].as_str().map(|s| s.to_string()),
                    client_email: data["client_email"].as_str().map(|s| s.to_string()),
                    client_phone: data["client_phone"].as_str().map(|s| s.to_string()),
                    date: data["created_date"]
                        .as_str()
                        .map_or(String::new(), ToString::to_string),
                    total_amount: data["total"].as_f64().map_or(0.0, |v| v),
                    tax: data["tax"].as_f64(),
                    notes: data["notes"].as_str().map(|s| s.to_string()),
                    regenerated: Some(regen),
                },
                items
                    .into_iter()
                    .map(|item| crate::models::NewItem {
                        invoice_id: 0,
                        description: item.description,
                        quantity: item.quantity as i32,
                        unit_price: item.price,
                        total: (item.price * item.quantity as f64),
                    })
                    .collect::<Vec<crate::models::NewItem>>(),
            );
        }
        Err(e) => {
            print!("{}", format!("Error generating invoice: {}", e).red());
        }
    }

    Ok(())
}
