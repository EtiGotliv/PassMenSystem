use actix_web::web;
use crate::controllers::passwords_controller::*;

pub fn config(cfg: &mut web::ServiceConfig) {
        cfg.service(get_users_with_3_or_more_passwords);
        cfg.service(create_password);
        cfg.service(get_passwords);
        cfg.service(get_password);
        cfg.service(update_password);
        cfg.service(delete_password);
}
