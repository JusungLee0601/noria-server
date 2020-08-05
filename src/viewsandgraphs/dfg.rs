use std::fmt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use std::cell::{RefCell};
use petgraph::graph::Graph;
use serde_json::Value;

use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use crate::units::row::Row;
use crate::units::change::Change;
use crate::types::datatype::DataType;
use crate::types::changetype::ChangeType;
use crate::operators::operation::Operation;
use crate::operators::operation::Operation::Leafor;

// CURRENT GRAPH DOES NOT END CHANGE CHAIN EARLY, SIGNIFICANT EFFECT ON THROUGHPUT

//DataFlowGraph
//root_id_map: map of root_id's to their NodeIndexes
//leaf_id_vec: just a list of leaf ids, used for printing
#[derive(Debug)]
pub struct DataFlowGraph {
    pub(crate) data: Graph<RefCell<Operation>, ()>,
    root_id_map: HashMap<String, NodeIndex>,
    leaf_id_vec: Vec<NodeIndex>,
    path_subgraph_map: HashMap<String, String>,
}

//Displays DFG
impl fmt::Display for DataFlowGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for leaf_index in self.leaf_id_vec.clone() {
            let op_ref = self.data.node_weight(leaf_index).unwrap().borrow_mut();

            match &*op_ref {
                Operation::Leafor(leaf) => write!(f, "{:#?}", leaf.mat_view),
                _ => Ok(())
            };
        }

        Ok(())
    }
}

//DFG functions, unexposed
impl DataFlowGraph { 
    /// Returns a Row from any JSValue, preferably an array
    pub fn process_into_row(some_iterable: &JsValue)
            -> Result<Row, JsValue> {
        let mut row_vec = Vec::new();

        let iterator = js_sys::try_iter(some_iterable)?.ok_or_else(|| {
            "need to pass iterable JS values!"
        })?;

        for x in iterator {
            let x = x?;

            row_vec.push(DataType::from(x));
        }

        Ok(Row::new(row_vec))
    }
}

//DFG Functions, exposed
impl DataFlowGraph { 
    /// Returns DFG from JSON input
    pub fn new(json: String) -> DataFlowGraph {
        let mut data = Graph::new();
        let mut root_id_map = HashMap::new();
        let mut leaf_id_vec = Vec::new();
        
        let obj: Value = serde_json::from_str(&json).unwrap();

        let operators: Vec<Value> = serde_json::from_value(obj["operators"].clone()).unwrap();

        //Operator processing
        //Important to note that I'm allowing for cloning of operators. Mostly this clones small
        //bits of data like conditions and rows, but for Leaf this technically calls for cloning an
        //entire view. I'm hoping to allow this only because at this stage, the graph operators
        //technically have empty fields for state and Views. If JSON were to be sent with non-empty
        //initial graphs, then this would no longer be trivial. I did this to solve the move, but 
        //I'm almost sure there are better ways to solve this, but am too lazy currently to figure 
        //it out -.-
        
        for op_val in operators {
            let op: Operation = serde_json::from_value(op_val).unwrap();

            let index = data.add_node(RefCell::new(op.clone()));

            match op {
                Operation::Rootor(inner_op) => {    
                    root_id_map.insert(inner_op.root_id, index);
                    
                },
                Operation::Leafor(_inner_op) => {    
                    leaf_id_vec.push(index);  
                },
                _ => {
                    
                }
            }
        }     

        let edges: Vec<Value> = serde_json::from_value(obj["edges"].clone()).unwrap();
        
        for edge in &edges {
            let pi: usize = serde_json::from_value(edge["parentindex"].clone()).unwrap();
            let pni = NodeIndex::new(pi);
            let ci: usize = serde_json::from_value(edge["childindex"].clone()).unwrap();
            let cni = NodeIndex::new(ci);

            data.add_edge(pni, cni, {});
        } 

        let path_subgraph_map: HashMap<String, String> = serde_json::from_value(obj["path_map"].clone()).unwrap();

        DataFlowGraph { data, root_id_map, leaf_id_vec, path_subgraph_map }
    }

    /// Applies inserts and deletions sent to a specified Root, propogates them
    /// through graph relying on the recursive operator calls
    pub fn change_to_root_js(&self, root_string: String, row_chng: &JsValue) {
        let root_node_index = *(self.root_id_map.get(&root_string).unwrap());
        let mut root_op = self.data.node_weight(root_node_index).unwrap().borrow_mut();

        let row_chng_rust = match Self::process_into_row(&row_chng) {
            Ok(row) => row,
            Err(_err) => Row::new(Vec::new()),
        };  

        let change = Change::new(ChangeType::Insertion, vec![row_chng_rust]);
        let change_vec = vec![change];
        
        root_op.process_change(change_vec, self, root_node_index, root_node_index);
    }

    pub fn change_to_root_json(&self, root_string: String, row_chng_json: String) {
        let change: Change = serde_json::from_str(&row_chng_json).unwrap();

        let root_node_index = *(self.root_id_map.get(&root_string).unwrap());
        let mut root_op = self.data.node_weight(root_node_index.clone()).unwrap().borrow_mut();

        let change_vec = vec![change];

        root_op.process_change(change_vec, self, NodeIndex::new(1), root_node_index.clone());
    }

    pub fn read(&self, leaf_index: usize, key_string: String) -> String {
        let mut leaf_op = self.data.node_weight(NodeIndex::new(leaf_index)).unwrap().borrow_mut();
        let key: DataType = serde_json::from_str(&key_string).unwrap();

        match &*leaf_op {
            Leafor(leaf) => {
                let row = (*leaf).mat_view.table.get(&key).unwrap();
                let j = serde_json::to_string(&row);
                
                match j {
                    Ok(string) => string,
                    Err(_err) => "error".to_owned(),
                }
            }
            _ => "error".to_owned(),
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn node_count(&self) -> usize {
        self.data.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.data.edge_count()
    }

    pub fn leaf_counts(&self) -> Vec<usize> {
        let mut node_vec = Vec::new();

        for index in &self.leaf_id_vec {
            let leaf_ref = self.data.node_weight(*index).unwrap().borrow_mut();

            match &*leaf_ref {
                Leafor(leaf) => node_vec.push(leaf.mat_view.table.len()),
                _ => (),
            };
        }

        node_vec
    }
}