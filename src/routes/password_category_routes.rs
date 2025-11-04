
use actix_web::web;
use crate::controllers::password_category_controller::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_password_category);
    cfg.service(delete_password_category);
    cfg.service(get_all_password_categories);  
    cfg.service(get_users_with_com_passwords)
;
}

