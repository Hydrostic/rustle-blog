
use salvo::prelude::*;
use crate::utils::response::{NormalResponseGlobal::UnauthorizedStatus, normal_response};

#[handler]
pub fn auth_middleware(depot: &mut Depot, _res: &mut Response, ctrl: &mut FlowCtrl) {
    if let None = depot.session().unwrap().get::<i32>("user_id"){
        _ = normal_response(UnauthorizedStatus);
        ctrl.skip_rest();
    }
    
}