use crate::types::changetype::ChangeType;
use crate::units::row::Row;

//Change, typing shows ChangeType, batch holds multiple potential changes
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub typing: ChangeType,
    pub batch: Vec<Row>
}

//Change functions
impl Change {
    //constructor
    pub fn new(typing: ChangeType, batch: Vec<Row>) -> Change {
        Change { typing, batch }
    }
}
