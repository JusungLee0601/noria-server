#[macro_use] extern crate log;

#[macro_use]
extern crate serde_derive;

use std::net::TcpListener;
use std::thread::spawn;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tungstenite::Message;
use tungstenite::Message::Text;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

pub mod operators;
pub mod types;
pub mod units;
pub mod viewsandgraphs;

use crate::types::operatortype::OperatorType;
use crate::types::operatortype::OperatorType::{A, I, L, P, R, S};
use crate::viewsandgraphs::dfg::DataFlowGraph;
use crate::units::serverchange::ServerChange;


// SOME NOTES

// Some key differences between the server vs clientside graph. First, because the serde was for
// sending graphs to the clientside graphs, we technically don't need to be able to string convert
// for serverside structures. It's also impossible to do because you can't serialize and clone
// the Websocket connection. Instead, I'll have to manually build the petgraphs, which isn't too 
// difficult. 



fn build_server_graph() -> DataFlowGraph {
    let mut graph = DataFlowGraph::new();

    let mut latency_test_subgraph = r##"{
        "operators": [
                {
                    "t": "Rootor",
                    "c": {
                        "root_id": "JoinLeft"
                    }
                },
                {
                    "t": "Rootor",
                    "c": {
                        "root_id": "JoinRight"
                    }
                },
                {
                    "t": "InnerJoinor",
                    "c": {
                        "parent_ids": [0, 1],
                        "join_cols": [1, 0]
                    }
                },
                {
                    "t": "Leafor",
                    "c": {
                        "mat_view": {
                            "name": "Users and VoteCounts",
                            "column_names": ["AuthorUserID", "StoryID", "StoryVoteCount"],
                            "schema": ["Int", "Int", "Int"],
                            "key_index": 1
                        }
                    }
                }
            ],
        "edges": [{
            "parentindex": 0,
            "childindex": 2
        }, {
            "parentindex": 1,
            "childindex": 2
        },
        {
            "parentindex": 2,
            "childindex": 3
        }]
    }"##;

    graph.add_path("/latencytest".to_owned(), latency_test_subgraph.to_owned());
    graph.add_path("/latencytestdummy".to_owned(), "".to_owned());

    let stories_root = r##"{
        "root_id": "Stories",
        "key_index": 1
    }"##;
    let votes_root = r##"{
        "root_id": "Votes",
        "key_index": 2
    }"##;
    let aggregator = r##"{
        "group_by_col": [0]
    }"##;

    graph.add_node(OperatorType::R, stories_root.to_owned());
    graph.add_node(OperatorType::R, votes_root.to_owned());
    graph.add_node(OperatorType::A, aggregator.to_owned());
    graph.add_leaf("JoinLeft".to_owned(), 1);
    graph.add_leaf("JoinRight".to_owned(), 0);

    graph.add_edge(0, 3);
    graph.add_edge(1, 2);
    graph.add_edge(2, 4);

    graph
}

fn main() {
    let graph = Arc::new(Mutex::new(build_server_graph()));

    println!("creating websocket");
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();

    for stream in server.incoming() {
        spawn(move || {
            let mut strings: Vec<String> = Vec::new();

            let callback = |req: &Request, mut response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                strings.push(req.uri().path().to_string().clone());

                println!("The request's headers are:");
                for (ref header, _value) in req.headers() {
                    println!("* {}", header);
                }

                // Let's add an additional header to our response to the client.
                let headers = response.headers_mut();
                headers.append("MyCustomHeader", ":)".parse().unwrap());
                headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                Ok(response)
            };

            let graph_ref = graph.clone();
            let mut g = graph_ref.lock().unwrap();

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let client_subgraph = g.path_subgraph_map.get(&strings[0]).unwrap();

            println!("{}", strings[0]);

            let graph_msg = Message::text(client_subgraph);
            websocket.write_message(graph_msg).unwrap();
            println!("Sending initial graph");
            
            loop {
                let msg = websocket.read_message().unwrap();
                if msg.is_binary() || msg.is_text() {
                    if let Text(inner_json) = msg {
                        let sc: ServerChange = serde_json::from_str(&inner_json).unwrap();
                        g.change_to_root(sc.root_id, sc.changes);
                    }
                }
            }
        });
    }
}