use std::fmt;
use std::collections::HashMap;

use crate::types::changetype::ChangeType;
use crate::types::datatype::DataType;
use crate::types::schematype::SchemaType;
use crate::units::row::Row;
use crate::units::change::Change;
use wasm_bindgen::prelude::*;

fn return_hash_v() -> HashMap<DataType, Row> {
    HashMap::new()
}

//View
//name: string name, assumed unique
#[wasm_bindgen]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct View {
    name: String,
    column_names: Vec<String>,
    schema: Vec<SchemaType>,
    key_index: usize,
    #[serde(default = "return_hash_v")]
    pub(crate) table: HashMap<DataType, Row>,
}

//displays View
impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name);
        for strings in self.column_names.iter() {
            write!(f, "{}", strings);
        }
        for (_key, row) in self.table.iter() {
            write!(f, "{:#?} \n", row);
        }

        //write!(f, "{:#?}", self)

        Ok(())
    }
}

//View functions, unexposed
impl View {
    /// Returns View assuming empty table
    pub fn newJSON(name: String, key_index: usize, column_names: Vec<String>, 
        schema: Vec<SchemaType>) -> View {
        let table = HashMap::new();

        View {name, key_index, column_names, schema, table}
    }

    /// Changes View's table given a vector of Changes
    pub fn change_table(&mut self, change_vec: Vec<Change>) {
        for change in &change_vec {
            for row in &change.batch {
                match change.typing {
                    ChangeType::Insertion => {
                        let key = row.data[self.key_index].clone();
                        self.table.insert(key, row.clone());
                    },
                    ChangeType::Deletion => {
                        let key = row.data[self.key_index].clone();
                        self.table.remove(&key);
                    },
                }
            }
        }
    }
}

//View functions, exposed
#[wasm_bindgen]
impl View {
    /// Returns View as a String
    pub fn render(&self) -> String {
        self.to_string()
    }
}
