use actix_web::web;
use crate::controllers::password_history_controller::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_password_history);
    cfg.service(get_password_history);
}
