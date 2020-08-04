use std::fmt;
use crate::types::datatype::DataType;

//Row, allows 2d representation in tables 
#[derive(Debug)]
#[derive(Hash, Eq, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Row {
    pub data: Vec<DataType>
}

//display Rows
impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // for datum in self.data.iter() {
        //     write!(f, "{} \n", datum);
        // }

        write!(f, "{:#?}", self)
    }
}

//Row functions 
impl Row {
    //constructor
    pub fn new(data: Vec<DataType>) -> Row {
        Row{ data }
    }

    //updates index
    pub fn update_index(&mut self, index: usize, update: DataType) {
        self.data[index] = update;
    }
}