mod client;
mod blockchain;
mod balance_table;
mod network;
mod utils;
mod lamport;

use client::Client;
use std::env;
use std::process;

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the client ID is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <client_id>", args[0]);
        process::exit(1);
    }

    // Parse the client ID from the command-line argument
    let client_id: u64 = match args[1].parse() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid client ID. Please provide a valid number.");
            process::exit(1);
        }
    };

    let mut client = Client::new(client_id);
    client.run();
}