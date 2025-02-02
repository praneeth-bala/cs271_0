use crate::utils;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub from: u64,
    pub to: u64,
    pub amt: i64,
    pub hash_pointer: String,
}

pub struct Blockchain {
    blocks: Vec<Block>,
    pub next_block: Block,
    pub ready: bool,
}

impl Blockchain {
    pub fn new() -> Self {
        Self { blocks: vec![], next_block: Block{from:0,to:0,amt:0,hash_pointer:"".to_string()}, ready: false }
    }

    pub fn ready_block(&mut self, from: u64, to: u64, amt: i64) {
        self.next_block.from = from;
        self.next_block.to = to;
        self.next_block.amt = amt;
        self.ready = true;
    }

    pub fn create_block(&mut self) -> Block {
        let previous_hash;
        if self.blocks.len() != 0 {
            previous_hash = self.blocks.get(self.blocks.len()-1).unwrap().hash_pointer.clone();
        } else {
            previous_hash = String::new();
        }
        let hash_pointer = utils::calculate_hash(&op_to_str(self.next_block.from, self.next_block.to, self.next_block.amt), &previous_hash);

        let block = Block {
            from: self.next_block.from,
            to: self.next_block.to,
            amt: self.next_block.amt,
            hash_pointer,
        };
        self.blocks.push(block.clone());
        self.ready=false;
        block
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn print_blockchain(&self) {
        println!("Blockchain:");
        for (index, block) in self.blocks.iter().enumerate() {
            println!("Block {}: Operation: {}, Hash Pointer: {}", index, op_to_str(block.from, block.to, block.amt), block.hash_pointer);
        }
    }
}

fn op_to_str(from: u64, to: u64, amt: i64) -> String {
    format!("{from}->{to} amt {amt}")
}