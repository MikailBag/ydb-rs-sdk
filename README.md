# Rust YDB SDK 
[![Latest Version](https://img.shields.io/crates/v/ydb.svg)](https://crates.io/crates/ydb) 
[![Released API docs](https://docs.rs/ydb/badge.svg)](https://docs.rs/ydb)
[![Unit tests](https://github.com/ydb-platform/ydb-rs-sdk/actions/workflows/rust-unit.yml/badge.svg)](https://github.com/ydb-platform/ydb-rs-sdk/actions/workflows/rust-unit.yml)

Rust SDK for YDB.


Integration tests, with dependency from real YDB database marked as ignored.
To run it:
1. Set YDB_CONNECTION_STRING env
2. run cargo test -- --include-ignored

# Example
```rust
use ydb::{ClientBuilder, Query, StaticToken, YdbResult};

#[tokio::main]
async fn main() -> YdbResult<()> {

 // create the driver
 let client = ClientBuilder::from_str("grpc://localhost:2136?database=local")?
    .with_credentials(StaticToken::from("asd"))
    .client()?;

 // wait until the background initialization of the driver finishes
 client.wait().await?;

 // read the query result
 let sum: i32 = client
    .table_client() // create table client
    .retry_transaction(|mut t| async move {
        // the code in transaction can retry a few times if there was a retriable error

        // send the query to the database
        let res = t.query(Query::from("SELECT 1 + 1 AS sum")).await?;

        // read exactly one result from the db
        let field_val: i32 = res.into_only_row()?.remove_field_by_name("sum")?.try_into()?;

        // return result
        return Ok(field_val);
    })
    .await?;

 // this will print "sum: 2"
 println!("sum: {}", sum);
    return Ok(());
}
```

# More examples
[Url shorneter application](https://github.com/ydb-platform/ydb-rs-sdk/tree/master/ydb-example-urlshortener)

[Many small examples](https://github.com/ydb-platform/ydb-rs-sdk/tree/master/ydb/examples)
