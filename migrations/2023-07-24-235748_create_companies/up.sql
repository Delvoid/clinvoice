CREATE TABLE companies (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    address VARCHAR,
    email VARCHAR,
    phone VARCHAR 
);

CREATE TABLE clients (
    id INTEGER  NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    address VARCHAR,
    email VARCHAR,
    phone VARCHAR  
);

CREATE TABLE company_clients (
    company_id INTEGER NOT NULL,
    client_id INTEGER NOT NULL,
    FOREIGN KEY(company_id) REFERENCES companies(id),
    FOREIGN KEY(client_id) REFERENCES clients(id),
    PRIMARY KEY(company_id, client_id)
);


CREATE TABLE invoices (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    company_id INTEGER,
    company_name VARCHAR NOT NULL,
    company_address VARCHAR,
    company_email VARCHAR,
    company_phone VARCHAR,
    client_id INTEGER,
    client_name VARCHAR NOT NULL,
    client_address VARCHAR,
    client_email VARCHAR,
    client_phone VARCHAR,
    date TEXT NOT NULL,
    total_amount DOUBLE  NOT NULL,
    logo_url VARCHAR,
    tax DOUBLE,
    notes TEXT,
    regenerated BOOLEAN DEFAULT FALSE,

    FOREIGN KEY(company_id) REFERENCES companies(id),
    FOREIGN KEY(client_id) REFERENCES clients(id)
);

CREATE TABLE items (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    description VARCHAR NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DOUBLE NOT NULL,
    total DOUBLE NOT NULL,

    FOREIGN KEY(invoice_id) REFERENCES invoices(id)
);