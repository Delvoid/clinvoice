<div id="top"></div>

<div align="center">

# Invoice Generation Tool
 <p align="center">
    <a href="https://github.com/Delvoid/Clinvoice/issues">Report Bug</a>
    ·
    <a href="https://github.com/Delvoid/Clinvoice/issues">Request Feature</a>
  </p>

</div>

![Banner](./assets/CLInvoice_banner.png)

A fast and user-friendly command line invoice generator written in Rust. Designed to simplify and automate invoice creation for freelancers and small businesses.

This CLI tool was created to explore building practical CLI applications in Rust while providing a useful productivity tool. It eliminates the hassle of manual invoice creation by generating customized PDF invoices right from the command line.


### Features

- Create and manage companies, clients, invoices
- Generate PDF invoices
- Invoice regeneration
- Search for existing records
- Built-in SQLite database

### Installation

#### From source

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone the repository
3. Run `cargo build --release`

```bash
git clone https://github.com/delvoid/clinvoice.git
cd cli-invoice
cargo build --release
```

#### From Precompiled Binaries

Precompiled binaries for Mac OS (Mach-O 64-bit executable arm64) and Windows (x86_64-pc-windows-gnu) are available in the [releases](https://github.com/delvoid/clinvoice/releases)
 section.

Download the appropriate binary for your system, add it to your PATH, and run:
    
```bash
cli-invoice
```


### Commands

#### Setup

Initialize the configuration and database. Creates config.json and a SQLite database file. This command runs on first launch and is required to run the app.

```bash
clinvoice setup
```

<p align="right">(<a href="#top">back to top</a>)</p>


#### Companies

The company details are your own details. These are used to generate the invoice.

```bash
# Create a company
cli-invoice company add # Prompts for details
# OR
cli-invoice company add [NAME] [ADDRESS] [EMAIL] [PHONE]

# Search companies
cli-invoice company search {name}
```

use `cli-invoice company --help` for more details.

#### Clients

The client details are the details of the person or company you are invoicing.

```bash
# Create a client
cli-invoice client add # Prompts for details
# OR
cli-invoice client add [NAME] [ADDRESS] [EMAIL] [PHONE]

# List clients
cli-invoice client list

# Search clients
cli-invoice client list --name <name>
# use cli-invoice client list --help for more options
```

use `cli-invoice client --help` for more details.

<p align="right">(<a href="#top">back to top</a>)</p>


![Client add prompts](./assets/client-add-prompt.gif)
![Client add options](./assets/client-add-options.gif)
![Client list example](./assets/client-list.gif)

<p align="right">(<a href="#top">back to top</a>)</p>



#### Invoices

Generate invoices for clients. Invoices are generated as PDF files. You are able to use prompts or options to generate invoices. Any missing details will be prompted for.

When generating an invoice, the company is optional. By default, the first company in the database will be used. If you have multiple companies, you can specify which company to use by passing the `--company` option.

When searching for invoice, if there is more then one result it will show all results without items. You can then use the `--id` option to get the full invoice with items.

```bash
# Generate an invoice through prompts
cli-invoice invoice generate

# Generate an invoice through options 
cli-invoice invoice generate  --client John Doe --item '{"description": "Service 1", "quantity": 1, "price": 50}'

# List invoices
cli-invoice invoice list

# Search invoices
cli-invoice invoice search --client John Doe

# Regenerate an invoicei - creates a new invoice with the same details
cli-invoice invoice regen --id 1234
```
see `cli-invoice invoice --help` for more details.

<p align="right">(<a href="#top">back to top</a>)</p>


![Invoice options](./assets/invoice-options.gif)
![Invoice List by id](./assets/invoice-list-id.gif)

<p align="right">(<a href="#top">back to top</a>)</p>


 

### Database

The app uses an embedded SQLite database to store invoice data. The database file is created on initial setup at `<User home dir>/clinvoice/cli_invoice.sqlite3.`

### PDF Generation

Invoice PDFs are generated using [Handlebars](https://github.com/sunng87/handlebars-rust) templates and [Headless Chrome](https://github.com/rust-headless-chrome/rust-headless-chrome) via the [headless_chrome](https://crates.io/crates/headless_chrome) crate.

Template options are currently a work in progress.


### Contributions

Contributions are welcome! Feel free to open an issue or submit a pull request.

## Contact

Delvoid - [@delvoid](https://twitter.com/delvoid)


<p align="right">(<a href="#top">back to top</a>)</p>
