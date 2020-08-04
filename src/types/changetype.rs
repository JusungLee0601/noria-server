//Change, delineates Insertion vs Deletion
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ChangeType {
    Insertion,
    Deletion
}