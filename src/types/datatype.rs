use std::fmt;
use wasm_bindgen::JsValue;

//Data
#[derive(Debug)]
#[derive(Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum DataType {
    None,
    Int(i32),
    Text(String)
}

//from conversion, JsValue->DataType
impl From<JsValue> for DataType {
    fn from(item: JsValue) -> Self {
        if (item).as_f64().is_some()  {
            DataType::Int(item.as_f64().unwrap() as i32)
        } else if ( item).as_string().is_some()  {
            DataType::Text(item.as_string().unwrap())
        } else {
            DataType::None
        }
    }
}

//from conversion, f64->SchemaType
impl From<&JsValue> for DataType {
    fn from(item: &JsValue) -> Self {
        if (*item).as_f64().is_some()  {
            DataType::Int(item.as_f64().unwrap() as i32)
        } else if (*item).as_string().is_some()  {
            DataType::Text(item.as_string().unwrap())
        } else {
            DataType::None
        }
    }
}

//displays DataTypes
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::None => write!(f, "*"),
            DataType::Text(n) => {
                write!(f, "{}", n)
            }
            DataType::Int(n) => write!(f, "{}", n)
        }
    }
}