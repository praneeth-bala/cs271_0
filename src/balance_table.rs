use std::collections::HashMap;

pub struct BalanceTable {
    pub balances: HashMap<u64, i64>,
}

impl BalanceTable {
    pub fn new() -> Self {
        let balances = HashMap::new();
        Self { balances }
    }

    pub fn update_balance(&mut self, client: u64, amount: i64) {
        if let Some(balance) = self.balances.get_mut(&client) {
            *balance += amount;
        }
    }

    pub fn print_table(&self) {
        println!("Balance Table:");
        for (client, balance) in &self.balances {
            println!("{}: ${}", client, balance);
        }
    }
}