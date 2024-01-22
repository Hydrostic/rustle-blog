
use salvo::prelude::*;
use crate::utils::response::render_error;
use crate::utils::error_handling::NormalErrorGlobal;

#[handler]
pub fn auth_middleware(depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    if let None = depot.session().unwrap().get::<i32>("user_id"){
        render_error(NormalErrorGlobal::UnauthorizedStatus.into(), depot, res);
        ctrl.skip_rest();
    }
    
}