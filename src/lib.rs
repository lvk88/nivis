use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C"{
    fn alert(msg: &str);
}

#[wasm_bindgen]
pub fn get_name() -> String{
    "Laszlo".into()
}

#[wasm_bindgen]
pub fn alert_user(){
    alert(&get_name()[..]);
}
