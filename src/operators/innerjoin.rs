use crate::units::row::Row;
use crate::units::change::Change;
use crate::types::changetype::ChangeType;
use crate::types::datatype::DataType;
use crate::viewsandgraphs::dfg::DataFlowGraph;
use petgraph::graph::NodeIndex;
use crate::operators::Operator;
use std::collections::HashMap;

fn return_hash_i() -> HashMap<DataType, Vec<Row>> {
    HashMap::new()
}

//hashmap sorted by joined row, but can't be unique :(
//using a vector of rows instead, keyed on the join columns for either left or right
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct InnerJoin {
    parent_ids: Vec<usize>,
    #[serde(default = "return_hash_i")]
    left_state: HashMap<DataType, Vec<Row>>,
    #[serde(default = "return_hash_i")]
    right_state: HashMap<DataType, Vec<Row>>,
    join_cols: Vec<usize>,
}

//maybe switch up views as well
impl Operator for InnerJoin {
    fn apply(&mut self, prev_change_vec: Vec<Change>) -> Vec<Change> {
        prev_change_vec
    }

    fn process_change(&mut self, change: Vec<Change>, dfg: &DataFlowGraph, parent_index: NodeIndex, self_index: NodeIndex) { 
        let next_change = self.apply_join(change, parent_index);
        let graph = &(*dfg).data;
        let neighbors_iterator = graph.neighbors(self_index);

        for child_index in neighbors_iterator {
            let child_cell = (*graph).node_weight(child_index).unwrap();
            let mut child_ref_mut = child_cell.write().unwrap();

            //the self become parent, child becomes self
            (*child_ref_mut).process_change(next_change.clone(), dfg, self_index, child_index);
        }
    }
}

impl InnerJoin {
    fn apply_join(&mut self, prev_change_vec: Vec<Change>, p_id: NodeIndex) -> Vec<Change> {
        //pid check for left vs right
        //in comparison to aggregate, don't think I need 'joined' state, because have to recheck and 
        //changes don't "multiply", all unique changes and all their relevant joins get consolidated
        //into one single change with a variety of vec<row>s in batch 
        //LEFT LOSES JOIN VAL, RIGHT KEEPS AND IS APPENDED
        let mut next_change_vec = Vec::new();

        if p_id.index() == self.parent_ids[0] {
            for change in prev_change_vec {
                match change.typing {
                    ChangeType::Insertion => {
                        let mut new_change_batch = Vec::new();

                        for row in &(change.batch) {
                            //first insert into left state
                            let join_val = row.data[self.join_cols[0]].clone();

                            //check to see if keyed value already exists
                            match self.left_state.get_mut(&join_val) {
                                None => {self.left_state.insert(join_val.clone(), vec![row.clone()]);},
                                Some(vec) => {(*vec).push(row.clone());},
                            }

                            match self.right_state.get_mut(&join_val) {
                                //no match, no changes downstream assuming excluded NULLS
                                None => (),
                                //group of matches, require downstream inserts
                                Some(vec) => {
                                    for right_row in vec {
                                        let mut ins_row = row.clone();
                                        ins_row.data.remove(self.join_cols[0]);
                                        ins_row.data.extend(right_row.clone().data);
                                        new_change_batch.push(ins_row);
                                    }
                                },
                            }
                        }
                        
                        let insert_new = Change::new(ChangeType::Insertion, new_change_batch);
                        next_change_vec.push(insert_new);
                    }
                    ChangeType::Deletion => {
                        let mut new_change_batch = Vec::new();

                        for row in &(change.batch) {
                            //remove from left state
                            let join_val = row.data[self.join_cols[0]].clone();

                            match self.left_state.get_mut(&join_val) {
                                //shouldn't happen, assumes deletion for an item that doesn't exist
                                None => (),
                                //vec of possible deletion matches, .remove_item should find if exists
                                Some(vec) => {
                                    let pos = vec.iter().position(|r| r == row).unwrap();
                                    (*vec).remove(pos);
                                },
                            }

                            //send deletions downstream
                            match self.right_state.get_mut(&join_val) {
                                //no match, no changes downstream assuming excluded NULLS
                                None => (),
                                //group of matches, require downstream deletes
                                Some(vec) => {
                                    for right_row in vec {
                                        let mut del_row = row.clone();
                                        del_row.data.remove(self.join_cols[0]);
                                        del_row.data.extend(right_row.clone().data);
                                        new_change_batch.push(del_row);
                                    }
                                }
                            }
                        }

                        let delete_new = Change::new(ChangeType::Deletion, new_change_batch);
                        next_change_vec.push(delete_new);
                    }
                }
            }
        } else {
            for change in prev_change_vec {
                match change.typing {
                    ChangeType::Insertion => {
                        let mut new_change_batch = Vec::new();

                        for row in &(change.batch) {
                            //first insert into right state
                            let join_val = row.data[self.join_cols[1]].clone();

                            //check to see if keyed value already exists
                            match self.right_state.get_mut(&join_val) {
                                None => {self.right_state.insert(join_val.clone(), vec![row.clone()]);},
                                Some(vec) => {(*vec).push(row.clone());},
                            }

                            match self.left_state.get_mut(&join_val) {
                                //no match, no changes downstream assuming excluded NULLS
                                None => (),
                                //group of matches, require downstream inserts
                                Some(vec) => {
                                    for left_row in vec {
                                        let mut ins_row = left_row.clone();
                                        ins_row.data.remove(self.join_cols[0]);
                                        ins_row.data.extend(row.clone().data);
                                        new_change_batch.push(ins_row);
                                    }
                                },
                            }
                        }
                        
                        let insert_new = Change::new(ChangeType::Insertion, new_change_batch);
                        next_change_vec.push(insert_new);
                    }
                    ChangeType::Deletion => {
                        let mut new_change_batch = Vec::new();

                        for row in &(change.batch) {
                            //remove from right state
                            let join_val = row.data[self.join_cols[1]].clone();

                            match self.right_state.get_mut(&join_val) {
                                //shouldn't happen, assumes deletion for an item that doesn't exist
                                None => (),
                                //vec of possible deletion matches, .remove_item should find if exists
                                Some(vec) => {
                                    let pos = vec.iter().position(|r| r == row).unwrap();
                                    (*vec).remove(pos);
                                },
                            }

                            //send deletions downstream
                            match self.left_state.get_mut(&join_val) {
                                //no match, no changes downstream assuming excluded NULLS
                                None => (),
                                //group of matches, require downstream deletes
                                Some(vec) => {
                                    for left_row in vec {
                                        let mut del_row = left_row.clone();
                                        del_row.data.remove(self.join_cols[0]);
                                        del_row.data.extend(row.clone().data);
                                        new_change_batch.push(del_row);
                                    }
                                }
                            }
                        }

                        let delete_new = Change::new(ChangeType::Deletion, new_change_batch);
                        next_change_vec.push(delete_new);
                    }
                }
            }
        }

        next_change_vec
    }
}