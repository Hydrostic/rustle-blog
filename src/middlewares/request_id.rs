use salvo::prelude::*;
use uuid::Uuid;

#[handler]
pub fn request_id_middleware(depot: &mut Depot, _res: &mut Response, _ctrl: &mut FlowCtrl) {
   depot.insert("request_id", Uuid::new_v4().to_string());
}