use crate::balance_table::BalanceTable;
use crate::blockchain::Blockchain;
use crate::lamport::LamportQueue;
use crate::network::{Message, Network};
use std::sync::mpsc::{self, Receiver, Sender};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread,
};

pub struct Resources {
    blockchain: Blockchain,
    balance_table: BalanceTable,
    network: Network,
    lamport_queue: LamportQueue,
}
pub struct Client {
    client_id: u64,
    resources: Arc<Mutex<Resources>>,
}

impl Client {
    pub fn new(client_id: u64) -> Self {
        let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        let cli = Self {
            client_id: client_id,
            resources: Arc::new(Mutex::new(Resources{
                blockchain: Blockchain::new(),
                balance_table: BalanceTable::new(),
                network: Network::new(sender),
                lamport_queue: LamportQueue::new(),
            })),
        };

        let resources_clone = Arc::clone(&cli.resources);
        thread::spawn(move || {
            handle_incoming_events(
                resources_clone,
                receiver,
                client_id,
            );
        });
        cli
    }

    fn setup_connections(&mut self) -> usize{

        let mut resources_lock = self.resources.lock();
        let resource = resources_lock.as_deref_mut().unwrap();

        loop {
            print!("Enter command (connect <id> <port> / listen <id> <port> / done): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.eq_ignore_ascii_case("done") {
                break;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 3 {
                println!(
                    "Invalid command. Please enter a command in the format: <action> <id> <port>"
                );
                continue;
            }

            let action = parts[0];
            let client_id: u64 = match parts[1].parse() {
                Ok(id) => id,
                Err(_) => {
                    println!("Invalid client ID. Please enter a valid number.");
                    continue;
                }
            };

            let port: u16 = match parts[2].parse() {
                Ok(p) => p,
                Err(_) => {
                    println!("Invalid port. Please enter a valid number.");
                    continue;
                }
            };

            match action.to_lowercase().as_str() {
                "connect" => resource.network.connect_to_peer(client_id, port),
                "listen" => resource.network.listen_for_peer(client_id, port),
                _ => { println!("Unknown command. Use 'connect' or 'listen'."); continue;},
            }
            resource.balance_table.balances.insert(client_id, 10);
        }
        let total = resource.network.peers.len();
        println!(
            "{total} Connected clients",
        );

        drop(resources_lock);

        total
    }

    pub fn run(&mut self) {
        let total_peers = self.setup_connections();

        loop {
            print!("Enter command (send <recipient_id> <amt> / balance / blockchain): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.eq_ignore_ascii_case("balance") {
                self.resources.lock().as_deref().unwrap().balance_table.print_table();
                continue;
            } else if input.eq_ignore_ascii_case("blockchain") {
                self.resources.lock().as_deref().unwrap().blockchain.print_blockchain();
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 3 {
                println!(
                    "Invalid command. Please enter a command in the format: send <recipient_id> <amt>"
                );
                continue;
            }

            let recipient_id: u64 = match parts[1].parse() {
                Ok(id) => id,
                Err(_) => {
                    println!("Invalid client ID. Please enter a valid number.");
                    continue;
                }
            };

            let amt: i64 = match parts[2].parse() {
                Ok(p) => p,
                Err(_) => {
                    println!("Invalid amt. Please enter a valid number.");
                    continue;
                }
            };

            let mut resources_lock = self.resources.lock();
            let resource = resources_lock.as_deref_mut().unwrap();

            resource.lamport_queue.increment();
            resource.lamport_queue.insert(resource.lamport_queue.get_clock(), self.client_id);
            resource.blockchain.ready_block(self.client_id, recipient_id, amt);
            for i in 0..total_peers+1 {
                if i as u64 == self.client_id {
                    continue;
                }
                resource.network.send_message(i.try_into().unwrap(), Message::Request { client_id: (self.client_id), lamport_clock: (resource.lamport_queue.get_clock()) });
            }

            drop(resources_lock);
        }
    }
}

fn handle_incoming_events(
    resources: Arc<Mutex<Resources>>,
    receiver: Receiver<Message>,
    my_client_id: u64,
) {
    let mut reply_count = 0;
    loop {
        match receiver.recv() {
            Ok(message) => {
                // println!("Main thread received message: {:?}", message);
                let mut resources_lock = resources.lock();
                let resource = resources_lock.as_deref_mut().unwrap();

                match message {
                    Message::Request {
                        client_id,
                        lamport_clock,
                    } => {
                        resource.lamport_queue.insert(lamport_clock, client_id);
                        resource.lamport_queue.update(lamport_clock);
                        resource.network.send_message(client_id, Message::Reply { client_id: (my_client_id), lamport_clock: (resource.lamport_queue.get_clock()) });
                        println!("Added request to queue: {:?}", message);
                    }
                    Message::Release {
                        client_id: _,
                        lamport_clock,
                        block,
                    } => {
                        resource.lamport_queue.pop();
                        resource.lamport_queue.update(lamport_clock);
                        resource.blockchain.add_block(block.clone());
                        resource.balance_table.update_balance(block.from, -block.amt);
                        resource.balance_table.update_balance(block.to, block.amt);
                        println!("Processed release and added block: {:?}", block);
                    }
                    Message::Reply {
                        client_id,
                        lamport_clock,
                    } => {
                        resource.lamport_queue.update(lamport_clock);
                        reply_count += 1;
                        println!("Received reply from: {}", client_id);
                    }
                }

                if (resource.network.peers.len()==reply_count) && ((!resource.lamport_queue.peek().is_none()) && resource.lamport_queue.peek().unwrap().client_id==my_client_id) {
                    let block = resource.blockchain.create_block();
                    resource.balance_table.update_balance(block.from, -block.amt);
                    resource.balance_table.update_balance(block.to, block.amt);
                    resource.lamport_queue.increment();
                    resource.lamport_queue.pop();
                    println!("Received all replies, added block: {:?}", block);
                    for i in 0..reply_count+1 {
                        if i as u64 == my_client_id {
                            continue;
                        }
                        resource.network.send_message(i.try_into().unwrap(), Message::Release { client_id: (my_client_id), lamport_clock: (resource.lamport_queue.get_clock()), block: {block.clone()} });
                    }
                    reply_count=0;
                }
                
                drop(resources_lock);
            }
            Err(_) => {
                println!("Channel closed");
                break;
            }
        }
    }
}
