#[derive(Debug)]
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum PermissionType {
    Read,
    Write,
}