use crate::units::change::Change;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use crate::viewsandgraphs::view::View;
use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use tungstenite::protocol::WebSocket;

//Leaf Operator
//stored view is what is "accessed" by JS
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Leaf {
    pub(crate) table: HashMap<DataType, Row>,
    sockets: Vec<&WebSocket>,
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
        self.apply(change);

        for ws in &self.sockets {
            websocket.write_message(msg).unwrap();
        }
    }
}