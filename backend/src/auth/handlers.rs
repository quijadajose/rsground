use actix_web::{get, post, web, HttpResponse, Responder};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;


#[derive(Deserialize, Debug)]
struct GitHubUser {
    pub(crate) login: String,
    pub(crate) name: Option<String>,
    pub(crate) avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct OAuthData {
    pub client: oauth2::basic::BasicClient,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/auth")]
pub async fn oauth(oauth: web::Data<OAuthData>) -> impl Responder {
    let (auth_url, _csrf_token) = oauth.client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

async fn fetch_github_user(access_token: &str) -> Result<GitHubUser, reqwest::Error> {
    let client = reqwest::Client::new();
    let user_url = "https://api.github.com/user";

    let res = client
        .get(user_url)
        .header("User-Agent", "actix-web-oauth2")
        .bearer_auth(access_token)
        .send()
        .await?;

    let user: GitHubUser = res.json().await?;
    Ok(user)
}

#[get("/auth/callback")]
async fn auth_callback(query: web::Query<AuthRequest>, oauth_data: web::Data<OAuthData>) -> impl Responder {
    let code = AuthorizationCode::new(query.code.clone());

    let token_result = oauth_data.client.exchange_code(code)  // Cambio aquí
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret();
            match fetch_github_user(access_token).await {
                Ok(github_user) => {
                    let expiration = Utc::now() + Duration::hours(12);
                    let claims = Claims {
                        sub: github_user.login.clone(),
                        exp: expiration.timestamp() as usize,
                    };
                    let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
                    let jwt = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_ref()))
                        .expect("Error al generar el token");

                    HttpResponse::Ok().json(serde_json::json!({
                        "jwt": jwt,
                        "username": github_user.login,
                        "name": github_user.name,
                        "avatar_url": github_user.avatar_url,
                    }))
                },
                Err(err) => HttpResponse::InternalServerError().body(format!("Error al obtener el usuario de GitHub: {:?}", err)),
            }
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("Error al intercambiar el código: {:?}", err)),
    }
}

#[derive(Deserialize)]
    struct GuestLoginRequest {
        guest_name: String,
    }

#[post("/login-guest")]
    async fn guest_jwt(body: web::Json<GuestLoginRequest>) -> impl Responder {
        let guest_name = &body.guest_name;
        let guest_uuid = Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::hours(12);
        let claims = Claims {
            sub: guest_uuid.clone(),
            exp: expiration.timestamp() as usize,
        };

        let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
        let jwt = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_ref()))
            .expect("Error al generar el token");

        HttpResponse::Ok().json(serde_json::json!({
            "jwt": jwt,
            "uuid": guest_uuid,
            "name": guest_name,
        }))
    }
