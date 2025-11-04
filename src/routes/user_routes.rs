use actix_web::web;
use crate::controllers::users_controller::{
    create_user,
    get_users,
    get_user,
    update_user,
    delete_user,
    login,
    get_users_created_in_range,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_users_created_in_range)
        .service(create_user)
        .service(get_users)
        .service(get_user)
        .service(update_user)
        .service(delete_user)
        .service(login)
        ;

    }
