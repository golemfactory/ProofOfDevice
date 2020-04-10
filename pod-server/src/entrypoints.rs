use super::AppData;
use crate::error::AppError;
use crate::models::{NewUser, User};

use diesel::prelude::*;

use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use futures::channel::oneshot;
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::Deserialize;
use std::env;
use tokio::task;
use tokio_diesel::AsyncRunQueryDsl;

fn pub_key_from_quote(_quote: &Quote) -> String {
    "0123456789abcdef".to_string()
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    login: String,
    quote: Quote,
    nonce: Option<Nonce>,
}

pub async fn register(
    info: web::Json<RegisterInfo>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    let login_ = info.login.clone();
    log::info!(
        "Received register request for user with login '{}'.",
        login_
    );
    log::debug!("Received data: {:?}", info);

    // Check if user is already registered.
    let result = users
        .filter(login.eq(login_.clone()))
        .limit(1)
        .load_async::<User>(&app_data.pool)
        .await?;
    log::debug!("Matching user records found: {:?}", result);

    if result.len() > 0 {
        log::info!("User already registered.");
        return Err(AppError::AlreadyRegistered);
    }

    let (tx, rx) = oneshot::channel();
    app_data.rxs.lock().await.insert(login_.clone(), rx);
    task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // Verify the provided data with IAS.
        let api_key = match env::var("POD_SERVER_API_KEY") {
            Ok(api_key) => api_key,
            Err(err) => {
                tx.send(Err(AppError::from(err))).unwrap();
                return;
            }
        };
        if let Err(err) = IasHandle::new(&api_key, None, None).and_then(|handle| {
            handle.verify_quote(&info.quote, info.nonce.as_ref(), None, None, None, None)
        }) {
            tx.send(Err(AppError::from(err))).unwrap();
            return;
        }

        // Insert user to the database.
        let pub_key_ = pub_key_from_quote(&info.quote);
        let new_user = NewUser {
            login: info.login.clone(),
            pub_key: pub_key_,
        };
        let conn = match app_data.pool.get() {
            Ok(conn) => conn,
            Err(err) => {
                tx.send(Err(AppError::from(err))).unwrap();
                return;
            }
        };
        let res = diesel::insert_into(users)
            .values(new_user)
            .execute(&conn)
            .map(|_| ())
            .map_err(AppError::from);
        tx.send(res).unwrap();
    });
    let response = HttpResponse::Accepted()
        .header(header::LOCATION, format!("/register/{}/status", &login_))
        .finish();
    Ok(response)
}

pub async fn register_status(
    path: web::Path<String>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    log::info!(
        "Received register status result for user with login '{}'",
        path
    );
    let mut rxs = app_data.rxs.lock().await;
    let rx = match rxs.get_mut(&*path) {
        Some(rx) => rx,
        None => {
            let res: Result<_, AppError> = Ok(HttpResponse::BadRequest().finish());
            return res;
        }
    };
    log::debug!("rx handle: {:?}", rx);
    let body = match rx.try_recv()? {
        None => serde_json::json!({"status": "in progress"}),
        Some(res) => {
            log::debug!("Registration finished with: {:?}", res);
            res?;
            serde_json::json!({"status": "done"})
        }
    };
    Ok(HttpResponse::Ok().json(body))
}
