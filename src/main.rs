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
use crate::operators::operation::Operation::Leafor;
use crate::types::changetype::ChangeType;
use crate::types::permissiontype::PermissionType;
use crate::units::change::Change;


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

    let mut dummy_test_subgraph =  r##"{
        "operators": [
                {
                    "t": "Rootor",
                    "c": {
                        "root_id": "Only"
                    }
                },
                {
                    "t": "Leafor",
                    "c": {
                        "mat_view": {
                            "name": "Articles and Votes",
                            "column_names": ["Author", "Votes"],
                            "schema": ["Text", "Int"],
                            "key_index": 0
                        }
                    }
                }
            ],
        "edges": [{
            "parentindex": 0,
            "childindex": 1
        }]
    }"##;

    graph.add_path("/latencytest".to_owned(), latency_test_subgraph.to_owned());
    graph.add_path("/latencytestdummy".to_owned(), "".to_owned());
    graph.add_path("/dummytest".to_owned(), dummy_test_subgraph.to_owned());
    graph.add_permission("/dummytest".to_string(), PermissionType::Write);
    graph.add_permission("/dummytestread".to_string(), PermissionType::Read);

    // let stories_root = r##"{
    //     "root_id": "Stories",
    //     "key_index": 1
    // }"##;
    // let votes_root = r##"{
    //     "root_id": "Votes",
    //     "key_index": 2
    // }"##;
    // let aggregator = r##"{
    //     "group_by_col": [0]
    // }"##;

    // graph.add_node(OperatorType::R, stories_root.to_owned());
    // graph.add_node(OperatorType::R, votes_root.to_owned());
    // graph.add_node(OperatorType::A, aggregator.to_owned());
    // graph.add_leaf("JoinLeft".to_owned(), 1);
    // graph.add_leaf("JoinRight".to_owned(), 0);

    // graph.add_edge(0, 3);
    // graph.add_edge(1, 2);
    // graph.add_edge(2, 4);

    let stories_root = r##"{
        "root_id": "OnlyServer",
        "key_index": 0
    }"##;

    graph.add_node(OperatorType::R, stories_root.to_owned());
    graph.add_leaf("Only".to_owned(), 0, "/dummytest".to_string());

    graph.add_edge(0, 1);

    graph
}

fn main() {
    let graph = Arc::new(Mutex::new(build_server_graph()));

    println!("creating websocket");
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();

    for stream in server.incoming() {
        let graph_ref = Arc::clone(&graph);

        spawn( move || {
            let mut g = graph_ref.lock().unwrap();
            let mut path: String = "".to_string();

            let callback = |req: &Request, mut response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                path = req.uri().path().to_string();

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

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            let permission = g.path_permission_map.get(&path).unwrap();

            println!("{}", path);

            match permission {
                PermissionType::Write => {
                    let client_subgraph = g.path_subgraph_map.get(&path).unwrap();
                    let ws_leaf_ni = g.path_leaf_map.get(&path).unwrap();
                    
                    let graph_msg = Message::text(client_subgraph);
                    websocket.write_message(graph_msg).unwrap();
                    println!("Sending initial graph");
    
                    let mut leaf_op = g.data.node_weight(*ws_leaf_ni).unwrap().write().unwrap();
    
                    if let Leafor(leaf) = &mut*leaf_op {
                        leaf.initial_connect(websocket);
                    }
                },
                PermissionType::Read => {
                    loop {
                        let msg = websocket.read_message().unwrap();
                        if msg.is_binary() || msg.is_text() {
                            if let Text(inner_json) = msg {
                                println!("message received");
                                let sc: ServerChange = serde_json::from_str(&inner_json).unwrap();
                                println!("message converted");
                                g.change_to_root(sc.root_id, sc.changes);
                                println!("root change");
                                println!("{}", g.render());
                            }
                        }
                    }
                },            
            }
        });
    }
}