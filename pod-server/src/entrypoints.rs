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

fn verify_quote_and_insert(
    quote: &Quote,
    nonce: Option<&Nonce>,
    login_: &str,
    app_data: web::Data<AppData>,
) -> Result<(), AppError> {
    use crate::schema::users::dsl::*;
    // Verify the provided data with IAS.
    let api_key = env::var("POD_SERVER_API_KEY")?;
    let handle = IasHandle::new(&api_key, None, None)?;
    handle.verify_quote(quote, nonce, None, None, None, None)?;
    // Insert user to the database.
    let pub_key_ = pub_key_from_quote(quote);
    let new_user = NewUser {
        login: login_.to_string(),
        pub_key: pub_key_,
    };
    let conn = app_data.pool.get()?;
    diesel::insert_into(users)
        .values(new_user)
        .execute(&conn)
        .map(|_| ())?;
    Ok(())
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
        let res = verify_quote_and_insert(&info.quote, info.nonce.as_ref(), &info.login, app_data);
        tx.send(res).unwrap()
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
