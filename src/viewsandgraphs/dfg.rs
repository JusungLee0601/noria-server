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
use crate::types::operatortype::OperatorType;
use crate::operators::operation::Operation;
use crate::operators::operation::Operation::Aggregator;
use crate::operators::operation::Operation::InnerJoinor;
use crate::operators::operation::Operation::Projector;
use crate::operators::operation::Operation::Selector;
use crate::operators::operation::Operation::Leafor;
use crate::operators::operation::Operation::Rootor;
use crate::operators::leaf::Leaf;
use crate::operators::root::Root;
use crate::operators::aggregation::Aggregation;
use crate::operators::selection::Selection;
use crate::operators::innerjoin::InnerJoin;
use crate::operators::projection::Projection;

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
                Operation::Leafor(leaf) => write!(f, "{:#?}", leaf.table),
                _ => Ok(())
            };
        }

        Ok(())
    }
}

//DFG Functions, exposed
impl DataFlowGraph { 
    pub fn new() -> DataFlowGraph {
        let mut data = Graph::new();
        let mut root_id_map = HashMap::new();
        let mut leaf_id_vec = Vec::new();
        let mut path_subgraph_map = HashMap::new();   

        DataFlowGraph { data, root_id_map, leaf_id_vec, path_subgraph_map }
    }

    pub fn change_to_root_json(&self, root_string: String, row_chng_json: String) {
        let change: Change = serde_json::from_str(&row_chng_json).unwrap();

        let root_node_index = *(self.root_id_map.get(&root_string).unwrap());
        let mut root_op = self.data.node_weight(root_node_index.clone()).unwrap().borrow_mut();

        let change_vec = vec![change];

        root_op.process_change(change_vec, self, NodeIndex::new(1), root_node_index.clone());
    }

    pub fn add_node(&mut self, op_type: OperatorType, json: String) {
        match op_type {
            OperatorType::A => {
                let op: Aggregation = serde_json::from_str(&json).unwrap();
                let index = self.data.add_node(RefCell::new(Aggregator(op)));
            }
            OperatorType::I => {
                let op: InnerJoin = serde_json::from_str(&json).unwrap();
                let index = self.data.add_node(RefCell::new(InnerJoinor(op)));
            }
            OperatorType::P => {
                let op: Projection = serde_json::from_str(&json).unwrap();
                let index = self.data.add_node(RefCell::new(Projector(op)));
            }
            OperatorType::S => {
                let op: Selection = serde_json::from_str(&json).unwrap();
                let index = self.data.add_node(RefCell::new(Selector(op)));
            }
            OperatorType::R => {
                let op: Root = serde_json::from_str(&json).unwrap();
                let ri = op.root_id.clone();

                let index = self.data.add_node(RefCell::new(Rootor(op)));
                self.root_id_map.insert(ri, index);
            },
            _ => {},
        }
    }

    pub fn add_leaf(&mut self, root_pair_id: String, key_index: usize) {
        let leaf = Leaf::new(root_pair_id, key_index);
        let index = self.data.add_node(RefCell::new(Leafor(leaf)));
        self.leaf_id_vec.push(index); 
    }

    pub fn add_edge(&mut self, pi: usize, ci: usize) {
        let pni = NodeIndex::new(pi);
        let cni = NodeIndex::new(ci);
        self.data.add_edge(pni, cni, {});
    }

    pub fn read(&self, leaf_index: usize, key_string: String) -> String {
        let mut leaf_op = self.data.node_weight(NodeIndex::new(leaf_index)).unwrap().borrow_mut();
        let key: DataType = serde_json::from_str(&key_string).unwrap();

        match &*leaf_op {
            Leafor(leaf) => {
                let row = (*leaf).table.get(&key).unwrap();
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
                Leafor(leaf) => node_vec.push(leaf.table.len()),
                _ => (),
            };
        }

        node_vec
    }
}