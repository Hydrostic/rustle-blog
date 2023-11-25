
use salvo::prelude::*;
use crate::utils::response::{ResponseUtil, NormalResponseGlobal::UnauthorizedStatus};

#[handler]
pub fn auth_middleware(depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    if let None = depot.session().unwrap().get::<i32>("user_id"){
        _ = res.normal_response(UnauthorizedStatus);
        ctrl.skip_rest();
    }
    
}