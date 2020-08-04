use crate::types::changetype::ChangeType;
use crate::units::change::Change;

//Change, typing shows ChangeType, batch holds multiple potential changes
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct ServerChange {
    pub root_id: String,
    pub changes: Vec<Change>,
}