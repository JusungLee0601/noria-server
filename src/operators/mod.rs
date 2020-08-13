pub mod aggregation;
pub mod innerjoin;
pub mod projection;
pub mod leaf;
pub mod root;
pub mod selection;
pub mod operation;

use crate::units::change::Change;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use petgraph::graph::NodeIndex;
use tungstenite::protocol::WebSocket;
use std::net::TcpStream;

//Operator trait
pub trait Operator {
    /// Returns Vec of Changes after operator conditions applied
    fn apply(&mut self, prev_change: Vec<Change>) -> Vec<Change>; 

    /// Takes a set of Changes and propogates the Changes recursively through nodes children
    /// calls apply to generate new Change to send downward
    fn process_change(&mut self, change: Vec<Change>, dfg: &DataFlowGraph, _parent_index: NodeIndex, self_index: NodeIndex) {  
        let next_change = self.apply(change);
        let graph = &(*dfg).data;
        let neighbors_iterator = graph.neighbors(self_index);

        for child_index in neighbors_iterator {
            let child_cell = (*graph).node_weight(child_index).unwrap();
            let mut child_ref_mut = child_cell.write().unwrap();

            child_ref_mut.process_change(next_change.clone(), dfg, self_index, child_index);
        }
    }

    fn initial_connect(&mut self, mut ws: WebSocket<TcpStream>) {}
}
