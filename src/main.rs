use aerolineas_rusticas::node::{self, Node};
use std::env;

fn main() {


    let args: Vec<String> = env::args().collect();

    let mut node = Node::new(&args[1]);
    
    node.connect_node();

}

//cargo run -- puerto