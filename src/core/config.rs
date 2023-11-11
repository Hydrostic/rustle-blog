use serde::{Deserialize, Serialize};
use std::env;
use lazy_static::lazy_static;

#[derive(Deserialize, Serialize)]
struct BaseConfig{
 
}
lazy_static! {
    pub static ref DEBUG_MODE: bool = !env::var("DEBUG_ENABLED").is_err();
}
// pub fn load_config(){
    
// }