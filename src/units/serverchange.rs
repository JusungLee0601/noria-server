use crate::types::changetype::ChangeType;
use crate::units::change::Change;

//Change, typing shows ChangeType, batch holds multiple potential changes
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct ServerChange {
    pub root_id: String,
    pub changes: Vec<Change>,
}

//Change functions
impl ServerChange {
    //constructor
    pub fn new(root_id: String, changes: Vec<Change>) -> ServerChange {
        ServerChange { root_id, changes }
    }
}