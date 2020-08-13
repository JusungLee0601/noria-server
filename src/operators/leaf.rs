use crate::units::change::Change;
use crate::units::row::Row;
use crate::units::serverchange::ServerChange;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use crate::viewsandgraphs::view::View;
use crate::types::changetype::ChangeType;
use crate::types::datatype::DataType;
use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use tungstenite::protocol::WebSocket;
use std::collections::HashMap;
use tungstenite::Message;
use tungstenite::stream::Stream;
use std::net::TcpStream;
use std::io::Read;
use std::cell::{RefCell};

fn return_hash_v() -> HashMap<DataType, Row> {
    HashMap::new()
}

fn return_vec_v() -> Vec<WebSocket<TcpStream>> {
    Vec::new()
}

//Leaf Operator
//stored view is what is "accessed" by JS
#[derive(Debug)]
pub struct Leaf {
    pub(crate) table: HashMap<DataType, Row>,
    pub(crate) sockets: Vec<WebSocket<TcpStream>>,
    pub(crate) root_pair_id: String,
    key_index: usize,
}

//Operator Trait for Leaf
impl Operator for Leaf {
    ///Apply doesn't actually modify Change, inserts into mat_view table, returns unchanged input
    fn apply(&mut self, prev_change_vec: Vec<Change>) -> Vec<Change> {
        for change in &prev_change_vec {
            for row in &change.batch {
                match change.typing {
                    ChangeType::Insertion => {
                        let key = row.data[self.key_index].clone();
                        self.table.insert(key, row.clone());
                    },
                    ChangeType::Deletion => {
                        let key = row.data[self.key_index].clone();
                        self.table.remove(&key);
                    },
                }
            }
        }

        Vec::new()
    }

    /// Doesn't apply to the rest of the operators as it is the Leaf
    fn process_change(&mut self, change: Vec<Change>, _dfg: &DataFlowGraph, _parent_index: NodeIndex, _self_index: NodeIndex) { 
        self.apply(change.clone());  

        let server_change = ServerChange::new(self.root_pair_id.clone(), change);

        for n in 0..self.sockets.len() {
            let msg = Message::text(serde_json::to_string(&server_change.clone()).unwrap());
            let ws = self.get_ws(n);
            ws.write_message(msg).unwrap();
        }
    }

    fn initial_connect(&mut self, mut ws: WebSocket<TcpStream>) {
        //handle ended connection, remove websocket from vec
        let mut batch = Vec::new();

        for (key, val) in &self.table {
            batch.push(val.clone());
        }

        let initial_change = Change::new(ChangeType::Insertion, batch);
        let init_sc = ServerChange::new(self.root_pair_id.clone(), vec![initial_change]);

        let msg = Message::text(serde_json::to_string(&init_sc).unwrap());
        ws.write_message(msg).unwrap();
        self.sockets.push(ws);
    }
}

impl Leaf {
    pub fn new(root_pair_id: String, key_index: usize) -> Leaf {
        let table = HashMap::new();
        let sockets = Vec::new();

        Leaf { table, sockets, root_pair_id, key_index }
    }

    pub fn get_ws(&mut self, index: usize) -> &mut WebSocket<TcpStream> {
        self.sockets.get_mut(index).unwrap()
    }
}