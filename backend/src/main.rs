use std::io::Result;
use actix_web::{main, web::{get, Data}, App, HttpServer};
use routes::sessions::{get_session_ws, post_new_session, put_session_code, put_session_crates, put_session_name, run_session_code};
use tokio::sync::broadcast::channel;
use utils::structures::state::AppState;

pub mod routes;
pub mod utils;

#[main]
async fn main() -> Result<()> {
    let (session_update_tx, _) = channel(100);
    let data = Data::new( AppState { session_update_tx } );

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(post_new_session)
            .route("/session", get().to(get_session_ws))
            .service(put_session_name)
            .service(put_session_code)
            .service(put_session_crates)
            .service(run_session_code)
    })
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}
