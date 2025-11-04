use actix_web::web;
use crate::controllers::categories_controller::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_category);
    cfg.service(get_categories);
    cfg.service(get_category);
    cfg.service(update_category);
    cfg.service(delete_category);
    cfg.service(search_categories);
}
