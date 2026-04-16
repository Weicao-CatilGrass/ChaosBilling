mod bill;
mod calc;
mod cli;
mod display;
mod error;
mod macros;
mod who;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    cli::entry().await
}
