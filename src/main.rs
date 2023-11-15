use std::fs::File;
use std::io::Read;
use std::path::Path;
use dotenv::dotenv;
use dotenv_codegen::dotenv;
use regex::Captures;

fn main() {
    println!("Hello, world!");

    dotenv().ok();

    let mut config = postgres::Config::new();
    config
        .user(dotenv!("DB_USER"))
        .password(dotenv!("DB_PASS"))
        .dbname(dotenv!("DB_NAME"))
        .host(dotenv!("DB_HOST"));

    let mut client = config.connect(postgres::NoTls).unwrap();

    // Run migration
    let migration_sql = read_sql_file("resources/migration.sql");
    client.batch_execute(&migration_sql).unwrap();

    // Insert data only once, if table is empty
    let count: i64 = client.query_one("SELECT COUNT(*) FROM customer", &[]).unwrap().get(0);
    if count == 0 {
        let data_sql = read_sql_file("resources/data.sql");
        client.batch_execute(&data_sql).unwrap();
    }

    {
        println!("Prepping good query");
        let sql = read_sql_file("resources/query_good.sql");
        let prepped = client.prepare(&sql).unwrap();
        // Columns will print
        // [
        //     Column {
        //         name: "id",
        //         type: Int4,
        //     },
        //     Column {
        //         name: "first_name",
        //         type: Varchar,
        //     },
        //     Column {
        //         name: "last_name",
        //         type: Varchar,
        //     },
        //     Column {
        //         name: "email",
        //         type: Varchar,
        //     },
        //     Column {
        //         name: "date_of_birth",
        //         type: Date,
        //     },
        //     Column {
        //         name: "age",
        //         type: Float8,
        //     },
        // ]
        dbg!(prepped.columns());
        // Params will print
        // [
        //     Int4,
        // ]
        dbg!(prepped.params());
    }

    {
        println!("Prepping bad query");
        let sql = read_sql_file("resources/query_bad.sql");
        let _ = client.prepare(&sql).unwrap();
        // The above will panic with error `message: "column \"age\" does not exist"`
    }
}

fn read_sql_file(path: impl AsRef<Path>) -> String {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Replace custom param syntax with normal postgres param syntax
    replace_sql_params(&contents)
}

fn replace_sql_params(sql: &str) -> String {
    let regex = regex::Regex::new(r"\$[a-zA-Z0-9_]+").unwrap();
    let mut counter = 1;

    regex.replace_all(&sql, |_: &Captures| {
        let replacement = format!("${}", counter);
        counter += 1;
        replacement
    }).to_string()
}