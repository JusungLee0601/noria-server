use wasm_bindgen::JsValue;

//Schema, for Views only
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum SchemaType {
    None,
    Int,
    Text
}

//from conversion, JsValue->SchemaType
impl From<JsValue> for SchemaType {
    fn from(item: JsValue) -> Self {
        if item == 2 {
            SchemaType::Text
        } else if item == 1 {
            SchemaType::Int
        } else {
            SchemaType::None
        }
    }
}