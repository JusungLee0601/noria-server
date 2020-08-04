use crate::units::row::Row;
use crate::units::change::Change;
use crate::operators::Operator;

//Projection Operator
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Projection {
    columns: Vec<usize>,
}

//Operator Trait for Projection
impl Operator for Projection {
    fn apply(&mut self, prev_change_vec: Vec<Change>) -> Vec<Change> {
        let mut next_change_vec = Vec::new();

        for change in prev_change_vec {
            let mut next_change = Change { typing: change.typing, batch: Vec::new()};

            for row in &(change.batch) {
                let mut changed_row = Row::new(Vec::new());

                for index in &self.columns {
                    changed_row.data.push(row.data[*index].clone());
                }

                next_change.batch.push(changed_row);
            }

            next_change_vec.push(next_change);
        }

        next_change_vec
    }
}