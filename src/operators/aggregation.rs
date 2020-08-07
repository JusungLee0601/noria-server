use crate::units::row::Row;
use crate::units::change::Change;
use crate::types::changetype::ChangeType;
use crate::types::datatype::DataType;
use crate::operators::Operator;

use std::collections::HashMap;

fn return_hash_a() -> HashMap<Vec<DataType>, Row> {
    HashMap::new()
}

//Aggregation Operator
//group_by_col is ordered lowest to highest
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Aggregation {
    group_by_col: Vec<usize>,
    //function: FuncType,
    #[serde(default = "return_hash_a")]
    state: HashMap<Vec<DataType>, Row>,
}

//Operator Trait for Aggregation
//implements hard coded length for count, no sum or func matching yet
//also does not group changes first, which would be a lot cleaner, but harder to implement
impl Operator for Aggregation {
    fn apply(&mut self, prev_change_vec: Vec<Change>) -> Vec<Change> {
        let mut next_change_vec = Vec::new();

        //multiple Insertions and Deletions
        for change in prev_change_vec {
            match change.typing {
                ChangeType::Insertion => {
                    //multiple rows in a single Change
                    for row in &(change.batch) {
                        //form key to access aggregates in state
                        let mut temp_key = Vec::new();
                        
                        for index in &self.group_by_col {
                            temp_key.push(row.data[*index].clone());
                        } 

                        match self.state.get_mut(&temp_key) {
                            None => {
                                //create new row to insert with only the group by columns
                                let mut new_row_vec = Vec::new();

                                for index in &self.group_by_col {
                                    new_row_vec.push(row.data[*index].clone());
                                } 

                                //copy for key in hashmap
                                let new_row_key = new_row_vec.clone();

                                //since its a new key, gets its own count
                                new_row_vec.push(DataType::Int(1));

                                let new_row = Row::new(new_row_vec);

                                //apply changes to operator's internal state
                                self.state.insert(new_row_key, new_row.clone());

                                let mut change_rows = Vec::new();
                                change_rows.push(new_row.clone());
                            
                                //send insertion change downstream
                                let new_group_change = Change::new(ChangeType::Insertion, change_rows);
                                next_change_vec.push(new_group_change); 
                            },
                            Some(row_to_incr) => {
                                //sends deletion change downstream
                                let mut change_rows_del = Vec::new();
                                change_rows_del.push(row_to_incr.clone());

                                let delete_old = Change::new(ChangeType::Deletion, change_rows_del);
                                next_change_vec.push(delete_old);

                                //increments count in state
                                let len = &row_to_incr.data.len();
                                let new_count = match &row_to_incr.data[len - 1] {
                                    DataType::Int(count) => count + 1,
                                    _ => 0,
                                };
                                row_to_incr.data[len - 1] = DataType::Int(new_count);

                                //sends insertion change downstream
                                let mut change_rows_ins = Vec::new();
                                change_rows_ins.push(row_to_incr.clone());

                                let insert_new = Change::new(ChangeType::Insertion, change_rows_ins);
                                next_change_vec.push(insert_new);
                            },
                        }
                    }
                }
                //In this model, we assume that deletions will always match with one aggregated row
                ChangeType::Deletion => {
                    //multiple rows in a single Change
                    for row in &(change.batch) {
                        let mut temp_key = Vec::new();
                        
                        for index in &self.group_by_col {
                            temp_key.push(row.data[*index].clone());
                        } 

                        match self.state.get_mut(&temp_key) {
                            Some(row_to_decr) => {
                                //sends deletion change downstream
                                let mut change_rows_del = Vec::new();
                                change_rows_del.push(row_to_decr.clone());

                                let delete_old = Change::new(ChangeType::Deletion, change_rows_del);
                                next_change_vec.push(delete_old);

                                //decrements count in state
                                let len = &row_to_decr.data.len();
                                let new_count = match &row_to_decr.data[len - 1] {
                                    DataType::Int(count) => count - 1,
                                    _ => 0,
                                };
                                row_to_decr.data[len - 1] = DataType::Int(new_count);

                                //sends insertion change downstream if not decremented to 0
                                if new_count > 0 {
                                    let mut change_rows_ins = Vec::new();
                                    change_rows_ins.push(row_to_decr.clone());

                                    let insert_new = Change::new(ChangeType::Insertion, change_rows_ins);
                                    next_change_vec.push(insert_new);
                                }
                            },
                            None => {}
                        }
                    }
                }
            }
        }

        next_change_vec
    }
}
