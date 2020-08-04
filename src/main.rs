#[macro_use] extern crate log;

#[macro_use]
extern crate serde_derive;

use std::net::TcpListener;
use std::thread::spawn;
use std::collections::HashMap;

use tungstenite::Message;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

pub mod operators;
pub mod types;
pub mod units;
pub mod viewsandgraphs;

fn build_graph() -> String {
    let graph_json = r##"{
        "operators": [
                {
                    "t": "Rootor",
                    "c": {
                        "root_id": "TestRoot"
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
            "childindex": 3
        }, {
            "parentindex": 1,
            "childindex": 2
        },
        {
            "parentindex": 2,
            "childindex": 3
        },
        {
            "parentindex": 3,
            "childindex": 4
        }]
    }"##;

    graph_json.to_string()
}

fn main() {
    println!("creating websocket");
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, mut response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
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

            //writes initial graph 
            let graph = r##"{
                "operators": [
                        {
                            "t": "Rootor",
                            "c": {
                                "root_id": "AuthorStory"
                            }
                        },
                        {
                            "t": "Rootor",
                            "c": {
                                "root_id": "StoryVoter"
                            }
                        },
                        {
                            "t": "Aggregator",
                            "c": {
                                "group_by_col": [0]
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
                    "childindex": 3
                }, {
                    "parentindex": 1,
                    "childindex": 2
                },
                {
                    "parentindex": 2,
                    "childindex": 3
                },
                {
                    "parentindex": 3,
                    "childindex": 4
                }]
            }"##;
            let graph_msg = Message::text(graph);
            websocket.write_message(graph_msg).unwrap();
            println!("Sending intial graph");
            
            loop {
                let msg = websocket.read_message().unwrap();
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(msg).unwrap();
                }
            }
        });
    }
}