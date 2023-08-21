mod client;
mod company;
mod company_clients;
mod config;
mod db;
mod invoice;
mod models;
mod pdf;
mod schema;
mod utils;

use clap::{Args, Parser, Subcommand};
use invoice::generate_invoice;
use text_colorizer::*;

use crate::invoice::InvoiceSearchField;

#[derive(Parser)]
#[command(name = "CLInvoice")]
#[command(author = "David Hough")]
#[command(version = "0.1.0")]
#[command(about = "Store clients and generate invoices", long_about = None)]
#[command(
    help_template = "{author} {about-section}Version: {version} \n\n {usage-heading} {usage} \n\n {all-args} \n {tab}"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    Setup,
    #[command(subcommand)]
    /// Manage companies
    Company(CompanyCommands),
    #[command(subcommand)]
    /// Manage clients
    Client(ClientCommands),
    #[command(subcommand)]
    /// Manage invoices
    Invoice(InvoiceCommands),
}

#[derive(Subcommand)]
enum CompanyCommands {
    /// Add a company
    Add(CompanyAddArgs),
}

#[derive(Subcommand)]
enum ClientCommands {
    /// Add a client
    Add(ClientAddArgs),
    /// #[arg(short, long)]
    List(ClientListArgs),
}

#[derive(Subcommand)]
enum InvoiceCommands {
    /// Generate an invoice
    Generate(InvoiceGenerateArgs),
    /// List invoices
    /// #[arg(short, long)]
    List(InvoiceListArgs),
    /// regenerate an invoice
    Regen(InvoiceRegenArgs),
}

#[derive(Args)]
struct CompanyAddArgs {
    /// Name of the company
    name: Option<String>,
    /// Address of the company
    address: Option<String>,
    /// Email of the company
    email: Option<String>,
    /// Phone number of the company
    phone: Option<String>,
}

#[derive(Args)]
struct ClientAddArgs {
    /// Name of the client
    name: Option<String>,
    /// Eddress of the client
    address: Option<String>,
    /// Email of the client
    email: Option<String>,
    /// Phone number of the client
    phone: Option<String>,
}

#[derive(Args)]
struct ClientListArgs {
    /// The name of the client
    #[arg(long, short)]
    name: Option<String>,
    /// The company of the client
    #[arg(long, short)]
    company: Option<String>,
    /// The address of the client
    #[arg(long, short)]
    address: Option<String>,
}

#[derive(Args)]
struct InvoiceListArgs {
    // Client id
    #[arg(long, short)]
    id: Option<i32>,
    /// client name
    #[arg(long, short)]
    client: Option<String>,
    /// company name
    #[arg(long)]
    company: Option<String>,
    /// client address
    #[arg(long, short)]
    address: Option<String>,
}

#[derive(Args)]
struct InvoiceRegenArgs {
    /// The id of the invoice to regenerate
    id: i32,
}

#[derive(Args)]
pub struct InvoiceGenerateArgs {
    /// The name of the company
    #[arg(long)]
    company_name: Option<String>,

    #[arg(long)]
    company_address: Option<String>,

    #[arg(long)]
    company_email: Option<String>,

    #[arg(long)]
    company_number: Option<String>,

    /// The name of the client
    #[arg(long)]
    client_name: Option<String>,

    #[arg(long)]
    client_address: Option<String>,

    #[arg(long)]
    client_email: Option<String>,

    #[arg(long)]
    client_number: Option<String>,

    /// The items for the invoice in JSON format
    #[arg(long)]
    item: Option<Vec<String>>,

    #[arg(long)]
    notes: Option<String>,

    #[arg(long, default_value = "false")]
    custom: Option<bool>,
}
fn main() {
    if !config::is_setup_done() {
        println!("{}", "Setup not done".red());
        config::setup();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            config::setup();
        }
        Commands::Company(company_command) => match company_command {
            CompanyCommands::Add(args) => {
                println!("Add Company");
                company::create_company_prompts(args.name, args.address, args.email, args.phone);
            }
        },
        Commands::Client(client_command) => match client_command {
            ClientCommands::Add(args) => {
                println!("Add Client");
                client::create_client_prompts(args.name, args.address, args.email, args.phone);
            }
            ClientCommands::List(args) => {
                let count = args.name.is_some() as i32
                    + args.address.is_some() as i32
                    + args.company.is_some() as i32;
                if count > 1 {
                    println!("Please provide only one of name, address, or company");
                    return;
                }

                if let Some(name) = args.name {
                    println!("Clients with name containing {}\n", name.green());
                    client::search_by_name(&name, None);
                } else if let Some(company) = args.company {
                    println!("Clients with company containing {}\n", company.green());
                    client::search_by_company(&company, None);
                } else if let Some(address) = args.address {
                    println!("Clients with address containing {}\n", address.green());
                    client::search_by_address(&address, None);
                } else {
                    println!("Full list of clients");
                    client::list_clients();
                }
            }
        },
        Commands::Invoice(invoice_command) => match invoice_command {
            InvoiceCommands::Generate(args) => {
                // count args used to determine if there are any args

                let count = args.company_name.is_some() as i32
                    + args.company_address.is_some() as i32
                    + args.company_email.is_some() as i32
                    + args.client_name.is_some() as i32
                    + args.client_address.is_some() as i32
                    + args.client_email.is_some() as i32
                    + args.item.is_some() as i32
                    + args.notes.is_some() as i32;

                let used_args = count > 0;
                let is_custom = args.custom.unwrap_or_default();
                match generate_invoice(args, used_args, is_custom) {
                    Ok(_) => println!("{}", "Invoice generation complete.".green()),
                    Err(e) => println!("Error: {}", e),
                }
            }

            InvoiceCommands::List(args) => {
                let count = args.id.is_some() as i32
                    + args.client.is_some() as i32
                    + args.address.is_some() as i32
                    + args.company.is_some() as i32;
                if count > 1 {
                    println!("Please provide only one of id, name, address, or company");
                    return;
                }

                if let Some(id) = args.id {
                    println!("Invoice with id {}\n", id.to_string().green());
                    invoice::search_by(InvoiceSearchField::Id, &id.to_string(), Some(true));
                } else if let Some(client) = args.client {
                    println!("Invoices with name containing {}\n", client.green());
                    invoice::search_by(InvoiceSearchField::Client, &client, Some(true));
                } else {
                    println!("Full list of invoices");
                    invoice::list_invoices();
                }
            }

            InvoiceCommands::Regen(args) => {
                println!("Regenerating invoice with id {}", args.id);
                match invoice::regenerate(args.id) {
                    Ok(_) => println!("{}", "Invoice regenerated".green()),
                    Err(e) => println!("Error: {}", e),
                };
            }
        },
    }
}
