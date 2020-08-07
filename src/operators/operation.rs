use super::aggregation::Aggregation;
use super::innerjoin::InnerJoin;
use super::projection::Projection;
use super::leaf::Leaf;
use super::root::Root;
use super::selection::Selection;
use crate::units::change::Change;
use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use std::io::Read;

//Operation Enum, used for typing
//I think this was originally for exposing operators to JS, but now that operator stuff is handled
//Rust side I'm not sure if this still needs to exist, I can give it a try to switch
#[derive(Debug)]
pub enum Operation {
    Selector(Selection),
    Projector(Projection),
    Aggregator(Aggregation),
    Rootor(Root),
    Leafor(Leaf),
    InnerJoinor(InnerJoin),
}

//Operator Trait for Operation Enum
impl Operator for Operation {
    fn apply(&mut self, prev_change: Vec<Change>) -> Vec<Change> { 
        match self {
            Operation::Selector(op) => op.apply(prev_change),
            Operation::Projector(op) => op.apply(prev_change),
            Operation::Aggregator(op) => op.apply(prev_change),
            Operation::Rootor(op) => op.apply(prev_change),
            Operation::Leafor(op) => op.apply(prev_change),
            Operation::InnerJoinor(op) => op.apply(prev_change),
        }
    }

    fn process_change(&mut self, change: Vec<Change>, dfg: &DataFlowGraph, parent_index: NodeIndex, self_index: NodeIndex) { 
        match self {
            Operation::Selector(op) => op.process_change(change, dfg, parent_index, self_index),
            Operation::Projector(op) => op.process_change(change, dfg, parent_index, self_index),
            Operation::Aggregator(op) => op.process_change(change, dfg, parent_index, self_index),
            Operation::Rootor(op) => op.process_change(change, dfg, parent_index, self_index),
            Operation::Leafor(op) => op.process_change(change, dfg, parent_index, self_index),
            Operation::InnerJoinor(op) => op.process_change(change, dfg, parent_index, self_index),
        }
    }
}