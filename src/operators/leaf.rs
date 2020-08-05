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
use std::io::Read;
use std::cell::{RefCell};

fn return_hash_v() -> HashMap<DataType, Row> {
    HashMap::new()
}

fn return_vec_v<T>() -> Vec<T> {
    Vec::new()
}

//Leaf Operator
//stored view is what is "accessed" by JS
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Leaf<R: Read> {
    #[serde(default = "return_hash_v")]
    pub(crate) table: HashMap<DataType, Row>,
    root_pair_id: String,
    #[serde(default = "return_vec_v")]
    sockets: Vec<RefCell<WebSocket<Stream<R, R>>>>,
}

//Operator Trait for Leaf
impl Operator for Leaf<Read> {
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
        self.apply(change);

        let server_change = ServerChange::new(self.root_pair_id, change);
        let msg = Message::text(server_change.to_string());

        for ws in &self.sockets {
            ws.write_message(msg).unwrap();
        }
    }
}

impl Leaf<Read> {
    pub fn initial_connect(&mut self, ws: &WebSocket<Stream<Read, Read>>) {
        //handle ended connection, remove websocket from vec
        let mut batch = Vec::new();

        for (key, val) in &self.table {
            batch.push(val);
        }

        let initial_change = Change::new(ChangeType::Insertion, batch);
        let init_sc = ServerChange::new(self.root_pair_id, vec![initial_change]);

        let msg = Message::text(init_sc.to_string());
        ws.write_message(msg).unwrap();
        self.sockets.push(ws);
    }
}