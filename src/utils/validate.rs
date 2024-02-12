use tracing::error;
use validator::ValidationErrors;
use crate::get_args;
use crate::types::err::AppError;
use crate::types::err::GlobalUserError::InvalidParameter;


impl From<ValidationErrors> for AppError{
    fn from(e: ValidationErrors) -> Self {
        if get_args!(debug) {
            error!("ValidationError: {}", e);
        }
        return InvalidParameter.into();
    }
}