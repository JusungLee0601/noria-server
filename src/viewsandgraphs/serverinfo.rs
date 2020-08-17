use std::collections::HashMap;
use crate::types::permissiontype::PermissionType;
use petgraph::graph::NodeIndex;

pub struct ServerInfo {
    pub(crate) path_subgraph_map: HashMap<String, String>,
    pub(crate) path_permission_map: HashMap<String, PermissionType>,
}

impl ServerInfo {
    pub fn new() -> ServerInfo {
        let mut path_subgraph_map = HashMap::new(); 
        let mut path_permission_map = HashMap::new(); 

        ServerInfo{ path_subgraph_map, path_permission_map }
    }

    pub fn add_path(&mut self, path: String, subgraph: String) {
        self.path_subgraph_map.insert(path, subgraph);
    }

    pub fn add_permission(&mut self, path: String, pt: PermissionType) {
        self.path_permission_map.insert(path, pt);
    }
}