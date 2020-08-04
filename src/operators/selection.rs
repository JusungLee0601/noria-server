use crate::units::change::Change;
use crate::operators::Operator;
use crate::types::datatype::DataType;

//Selection Operator
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Selection {
    col_ind: usize,
    condition: DataType,
}

//Operator Trait for Selection
impl Operator for Selection {
    fn apply(&mut self, prev_change_vec: Vec<Change>) -> Vec<Change> {
        let mut next_change_vec = Vec::new();

        for change in prev_change_vec {
            let mut next_change = Change {typing: change.typing, batch: Vec::new()};

            for row in &(change.batch) {
                if row.data[self.col_ind] == self.condition {
                    next_change.batch.push((*row).clone());
                }
            }

            next_change_vec.push(next_change);
        }

        next_change_vec
    }
}