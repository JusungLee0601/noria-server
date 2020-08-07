use crate::units::change::Change;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use std::collections::HashMap;
use crate::types::datatype::DataType;
use crate::units::row::Row;
use crate::types::changetype::ChangeType;

fn return_hash_v() -> HashMap<DataType, Row> {
    HashMap::new()
}

//Root Operator
//root_id assumed unique, used for NodeIndex mapping to find in graph
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Root {
    pub(crate) root_id: String,
    key_index: usize, 
    #[serde(default = "return_hash_v")]
    pub(crate) table: HashMap<DataType, Row>,
    
}

//Operator Trait for Root
impl Operator for Root {
    /// Identity, doesn't need to modify change as Root
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

    /// For Root, process change does not "apply"/change the initial set of Changes as it is the Root
    fn process_change(&mut self, change: Vec<Change>, dfg: &DataFlowGraph, _parent_index: NodeIndex, self_index: NodeIndex) { 
        self.apply(change.clone());
        
        let graph = &(*dfg).data;
        let neighbors_iterator = graph.neighbors(self_index); 

        for child_index in neighbors_iterator { 
            let child_cell = (*graph).node_weight(child_index).unwrap();
            let mut child_ref_mut = child_cell.borrow_mut();
 
            //the self become parent, child becomes self
            (*child_ref_mut).process_change(change.clone(), dfg, self_index   , child_index);
        }
    }
}