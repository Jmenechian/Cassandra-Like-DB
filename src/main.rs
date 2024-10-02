use aerolineas_rusticas::node::{self, Node};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "Fix" { //cargo run -- Fix limpia nodes.txt
        fs::write("nodes.txt", "").expect("Failed to clear nodes.txt");
        return;
    }

    let mut node = Node::new(&args[1]);
    node.connect_node();
}

//cargo run -- puerto