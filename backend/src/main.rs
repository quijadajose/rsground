use actix::prelude::*;
use actix_web::{
    body::EitherBody, dev::{Service, ServiceRequest, ServiceResponse, Transform}, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder
};
use actix_web_actors::ws;
use chrono::{Duration, Utc};
use cola::{Deletion, Replica, ReplicaId};
use dotenv::dotenv;
use env_logger;
use futures::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use log::{error, info};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl, TokenResponse,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ops::Range,
    sync::{Arc, Mutex},
    sync::atomic::{AtomicU64, Ordering},
};
use uuid::Uuid;

static REPLICA_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_unique_replica_id() -> u64 {
    REPLICA_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Deserialize, Debug)]
struct GitHubUser {
    login: String,
    name: Option<String>,
    avatar_url: String,
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

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

struct OAuthData {
    client: BasicClient,
}

#[get("/auth")]
async fn auth(oauth: web::Data<OAuthData>) -> impl Responder {
    let (auth_url, _csrf_token) = oauth.client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
}

#[get("/auth/callback")]
async fn auth_callback(query: web::Query<AuthRequest>, oauth: web::Data<OAuthData>) -> impl Responder {
    let code = AuthorizationCode::new(query.code.clone());

    let token_result = oauth.client.exchange_code(code)
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

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareMiddleware { service })
    }
}

pub struct JwtMiddlewareMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").to_string();
                    let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
                    let validation = Validation::default();
                    if decode::<Claims>(&token, &DecodingKey::from_secret(secret_key.as_ref()), &validation).is_ok() {
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            let res = fut.await?;
                            return Ok(res.map_into_left_body());
                        });
                    }
                }
            }
        }
        Box::pin(async {
            let res = HttpResponse::Unauthorized().finish().map_into_right_body();
            Ok(req.into_response(res))
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
enum AccessLevel {
    ReadOnly,
    Editor,
}

impl FileNode {
    fn as_directory_mut(&mut self) -> Option<&mut HashMap<String, FileNode>> {
        if let FileNode::Directory(ref mut dir) = self {
            Some(dir)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Action {
    Insertion { pos: usize, text: String, timestamp: u64 },
    Deletion { range_start: usize, range_end: usize, timestamp: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum FileNode {
    File,
    Directory(HashMap<String, FileNode>),
}

#[derive(Debug)]
struct Document {
    buffer: String,
    crdt: Replica,
    replica_id: ReplicaId,
    history: Vec<Action>,
    current_timestamp: u64,
}
struct Insertion {
    text: String,
    crdt: cola::Insertion,
}

impl Document {
    fn new<S: Into<String>>(text: S, replica_id: ReplicaId) -> Self {
        let buffer = text.into();
        let crdt = Replica::new(replica_id, buffer.len());
        info!("Creando nuevo documento con buffer: '{}'", buffer);
        Document { 
            buffer, 
            crdt,
            replica_id, // Guardamos el id original
            history: Vec::new(),
            current_timestamp: 0,
        }
    }
    
    fn fork(&self, new_replica_id: ReplicaId) -> Self {
        // Ahora podemos comparar con self.replica_id
        let new_replica_id = if new_replica_id == self.replica_id {
            generate_unique_replica_id()
        } else {
            new_replica_id
        };
        let crdt = self.crdt.fork(new_replica_id);
        Document {
            buffer: self.buffer.clone(),
            crdt,
            replica_id: new_replica_id,
            history: self.history.clone(),
            current_timestamp: self.current_timestamp,
        }
    }
    

    fn insert<S: Into<String>>(&mut self, insert_at: usize, text: S) -> Insertion {
        let text = text.into();
        info!("Insertando texto '{}' en la posición {} del documento", text, insert_at);
        self.buffer.insert_str(insert_at, &text);
        let insertion = self.crdt.inserted(insert_at, text.len());
        self.current_timestamp += 1;
        let action = Action::Insertion { 
            pos: insert_at, 
            text: text.clone(), 
            timestamp: self.current_timestamp 
        };
        self.history.push(action);
        info!("Buffer actualizado: '{}'", self.buffer);
        Insertion { text, crdt: insertion }
    }

    fn delete(&mut self, range: Range<usize>) -> Deletion {
        info!("Eliminando rango {}..{} en el documento", range.start, range.end);
        self.buffer.replace_range(range.clone(), "");
        let deletion = self.crdt.deleted(range.clone());
        self.current_timestamp += 1;
        let action = Action::Deletion { 
            range_start: range.start, 
            range_end: range.end, 
            timestamp: self.current_timestamp 
        };
        self.history.push(action);
        info!("Buffer después de eliminación: '{}'", self.buffer);
        deletion
    }

    fn get_operations_since(&self, last_timestamp: u64) -> Vec<Action> {
        info!("Obteniendo operaciones con timestamp > {}", last_timestamp);
        self.history
            .iter()
            .filter(|action| match action {
                Action::Insertion { timestamp, .. } => *timestamp > last_timestamp,
                Action::Deletion { timestamp, .. } => *timestamp > last_timestamp,
            })
            .cloned()
            .collect()
    }
}

#[derive(Debug)]
struct Project {
    id: Uuid,
    name: String,
    owner: String,
    files: FileNode,
    documents: HashMap<String, Document>,
    allowed_users: HashMap<String, AccessLevel>,
    pending_editor_requests: Vec<String>,
    is_public: bool,
    password: Option<String>,
}

impl Project {
    fn new(owner: String, id: Uuid, name: impl Into<String>, is_public: bool, password: Option<String>) -> Self {
        if !is_public && password.is_none() {
            panic!("Los proyectos privados requieren contraseña");
        }
        Project {
            id,
            name: name.into(),
            owner,
            files: FileNode::Directory(HashMap::new()),
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            pending_editor_requests: Vec::new(),
            is_public,
            password,
        }
    }
    fn permit_access(&mut self, username: String, access: AccessLevel) {
        self.allowed_users.insert(username, access);
    }
    fn get_file_mut(&mut self, file_name: &str) -> Option<&mut Document> {
        self.documents.get_mut(file_name)
    }

    fn add_file(&mut self, path: &str, document: Document) {
        let mut parts: Vec<&str> = path.split('/').collect();
        if let Some(file_name) = parts.pop() {
            let mut current = match &mut self.files {
                FileNode::Directory(dir) => dir,
                _ => panic!("La raíz del proyecto debe ser un directorio"),
            };

            for part in parts {
                current = current
                    .entry(part.to_string())
                    .or_insert_with(|| FileNode::Directory(HashMap::new()))
                    .as_directory_mut()
                    .expect("Se esperaba un directorio");
            }
            current.insert(file_name.to_string(), FileNode::File);
            self.documents.insert(path.to_string(), document);
        }
    }

    fn get_files(&self) -> &FileNode {
        &self.files
    }

    fn get_file(&self, path: &str) -> Option<&Document> {
        self.documents.get(path)
    }
}
struct ProjectManager {
    projects: HashMap<Uuid, Project>,
}

impl ProjectManager {
    fn new() -> Self {
        info!("Inicializando ProjectManager");
        ProjectManager {
            projects: HashMap::new(),
        }
    }

    fn add_project(&mut self, project: Project) {
        info!("Agregando proyecto {}", project.id);
        self.projects.insert(project.id, project);
    }

    fn get_project(&self, id: Uuid) -> Option<&Project> {
        self.projects.get(&id)
    }

    fn get_project_mut(&mut self, id: Uuid) -> Option<&mut Project> {
        self.projects.get_mut(&id)
    }
}

struct AppState {
    manager: Mutex<ProjectManager>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ClientMessage {
    CreateProject { project_id: Uuid, name: String, is_public: bool, password: Option<String> },
    JoinProject { project_id: Uuid, access: AccessLevel, password: Option<String> },
    Insert { project_id: Uuid, file: String, pos: usize, text: String },
    Delete { project_id: Uuid, file: String, range_start: usize, range_end: usize },
    Sync { project_id: Uuid, file: String, last_timestamp: u64 },
    GetProjectFiles { project_id: Uuid },
    GrantEditor { project_id: Uuid, user_id: Uuid },
    PermitAccess { project_id: Uuid, username: String, access: AccessLevel },
    ForkProject { project_id: Uuid },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ServerMessage {
    UserConnected { user_id: String },
    ProjectCreated { project_id: Uuid },
    SyncActions { project_id: Uuid, file: String, actions: Vec<Action> },
    Error { message: String },
    ProjectFiles { project_id: Uuid, files: FileNode },
    JoinedProject { project_id: Uuid },
    EditorRequestReceived { project_id: Uuid },
    NewEditorRequest { project_id: Uuid, user_id: Uuid },
    EditorGranted { project_id: Uuid },
    Update { project_id: Uuid, file: String, content: String },
    ProjectForked { old_project_id: Uuid, new_project_id: Uuid },
}

struct MyWs {
    state: Arc<AppState>,
    user_id: String, 
    access: Option<AccessLevel>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let welcome_msg = ServerMessage::UserConnected { user_id: self.user_id.clone() };
        let text = serde_json::to_string(&welcome_msg).unwrap();
        ctx.text(text);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                info!("Mensaje recibido: {}", text);
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let response = self.handle_client_message(client_msg);
                    if let Some(resp) = response {
                        let resp_text = serde_json::to_string(&resp).unwrap();
                        info!("Enviando respuesta: {}", resp_text);
                        ctx.text(resp_text);
                    }
                } else {
                    error!("Error al parsear mensaje JSON");
                    let err = ServerMessage::Error { message: "Mensaje inválido".into() };
                    let _ = ctx.text(serde_json::to_string(&err).unwrap());
                }
            },
            Ok(ws::Message::Ping(msg)) => {
                info!("Ping recibido");
                ctx.pong(&msg);
            },
            Ok(ws::Message::Close(reason)) => {
                info!("Cierre de conexión: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            },
            Err(e) => {
                error!("Error en el stream de WebSocket: {:?}", e);
                ctx.stop();
            },
            _ => (),
        }
    }
}

impl MyWs {
    fn handle_client_message(&mut self, msg: ClientMessage) -> Option<ServerMessage> {    
        let mut manager = self.state.manager.lock().unwrap_or_else(|e| e.into_inner());
        match msg {
            ClientMessage::CreateProject { project_id, name, is_public, password } => {
                let project = Project::new(self.user_id.clone(), project_id, name, is_public, password);
                manager.add_project(project);
                self.access = Some(AccessLevel::Editor);
                Some(ServerMessage::ProjectCreated { project_id })
            },
            ClientMessage::JoinProject { project_id, access, password } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if access == AccessLevel::ReadOnly && !project.is_public {
                        match (&project.password, password) {
                            (Some(ref actual), Some(proporcionada)) if *actual == proporcionada => {
                                self.access = Some(AccessLevel::ReadOnly);
                                return Some(ServerMessage::JoinedProject { project_id });
                            },
                            _ => {
                                return Some(ServerMessage::Error { message: "Contraseña incorrecta para proyecto privado".into() });
                            }
                        }
                    }
            if access == AccessLevel::Editor {
                        let user_str = self.user_id.to_string();
                        if !project.pending_editor_requests.contains(&user_str) {
                            project.pending_editor_requests.push(user_str);
                        }
                        return Some(ServerMessage::EditorRequestReceived { project_id });
                    }
                    self.access = Some(access);
                    return Some(ServerMessage::JoinedProject { project_id });
                }
                Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
            },
            
            ClientMessage::GrantEditor { project_id, user_id } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if project.owner != self.user_id {
                        return Some(ServerMessage::Error { message: "No tienes permisos para otorgar acceso de editor".into() });
                    }
                    if let Some(pos) = project.pending_editor_requests.iter().position(|id| id == &user_id.to_string()) {
                        project.pending_editor_requests.remove(pos);
                        info!("Permiso de editor otorgado a usuario {}", user_id);
                        return Some(ServerMessage::Update {
                            project_id,
                            file: String::from(""),
                            content: format!("Usuario {} ahora tiene permisos de editor", user_id),
                        });
                    } else {
                        return Some(ServerMessage::Error { message: "El usuario no tiene una solicitud pendiente".into() });
                    }
                }
                Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
            },
            ClientMessage::Insert { project_id, file, pos, text } => {
                if let Some(AccessLevel::ReadOnly) = self.access {
                    return Some(ServerMessage::Error { message: "Permiso denegado: modo solo lectura".into() });
                }
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        let _ = doc.insert(pos, text);
                        return Some(ServerMessage::Update {
                            project_id,
                            file,
                            content: doc.buffer.clone(),
                        });
                    } else {
                        let mut new_doc = Document::new("", 1);
                        let _ = new_doc.insert(pos, text);
                        project.add_file(&file, new_doc);
                        if let Some(doc) = project.get_file(&file) {
                            return Some(ServerMessage::Update {
                                project_id,
                                file,
                                content: doc.buffer.clone(),
                            });
                        }
                    }
                }
                Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
            },

            ClientMessage::Delete { project_id, file, range_start, range_end } => {
                if let Some(AccessLevel::ReadOnly) = self.access {
                    return Some(ServerMessage::Error { message: "Permiso denegado: modo solo lectura".into() });
                }
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        let _ = doc.delete(range_start..range_end);
                        return Some(ServerMessage::Update {
                            project_id,
                            file,
                            content: doc.buffer.clone(),
                        });
                    } else {
                        return Some(ServerMessage::Error { message: format!("Archivo '{}' no encontrado en el proyecto {}", file, project_id) });
                    }
                }
                Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
            },
            ClientMessage::Sync { project_id, file, last_timestamp } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        info!("Sync: Sincronizando archivo '{}' en proyecto {} a partir del timestamp {}", file, project_id, last_timestamp);
                        let actions = doc.get_operations_since(last_timestamp);
                        return Some(ServerMessage::SyncActions {
                            project_id,
                            file,
                            actions,
                        });
                    } else {
                        error!("Sync: Archivo '{}' no encontrado en proyecto {}", file, project_id);
                        return Some(ServerMessage::Error { message: format!("Archivo '{}' no encontrado en el proyecto {}", file, project_id) });
                    }
                }
                error!("Sync: Proyecto {} no encontrado", project_id);
                Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
            },
            ClientMessage::GetProjectFiles { project_id } => {
                if let Some(project) = manager.get_project(project_id) {
                    Some(ServerMessage::ProjectFiles { 
                        project_id, 
                        files: project.get_files().clone() // Retorna la estructura completa
                    })
                } else {
                    Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
                }
            },
            ClientMessage::PermitAccess { project_id, username, access } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if project.owner != self.user_id {
                        return Some(ServerMessage::Error { 
                            message: "Solo el creador del proyecto puede conceder permisos".into() 
                        });
                    }            
                    project.permit_access(username.clone(), access);
                    info!("Permisos actualizados para el usuario {}", username);
                    Some(ServerMessage::Update {
                        project_id,
                        file: String::new(),
                        content: format!("Permisos actualizados para el usuario {}", username),
                    })
                } else {
                    Some(ServerMessage::Error { message: format!("Proyecto {} no encontrado", project_id) })
                }
            },
            ClientMessage::ForkProject { project_id } => {
                if self.access.is_none() {
                    return Some(ServerMessage::Error {
                        message: "Acceso denegado: se requiere al menos acceso de lectura para forquear".into(),
                    });
                }
                if let Some(project) = manager.get_project(project_id) {
                    let new_project_id = Uuid::new_v4();
                    let new_replica_id = generate_unique_replica_id(); // Genera un ReplicaId único
                    let forked_documents: HashMap<String, Document> = project.documents.iter().map(|(path, doc)| {
                        (path.clone(), doc.fork(new_replica_id))
                    }).collect();
            
                    let forked_project = Project {
                        id: new_project_id,
                        name: format!("{} (fork)", project.name),
                        owner: self.user_id.clone(),
                        files: project.files.clone(),
                        documents: forked_documents,
                        allowed_users: HashMap::new(),
                        pending_editor_requests: Vec::new(),
                        is_public: project.is_public,
                        password: project.password.clone(),
                    };
            
                    manager.add_project(forked_project);
                    self.access = Some(AccessLevel::Editor);
                    return Some(ServerMessage::ProjectForked { old_project_id: project_id, new_project_id });
                } else {
                    return Some(ServerMessage::Error {
                        message: format!("Proyecto {} no encontrado", project_id),
                    });
                }
            }                    
        }
    }
}

#[get("/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<AppState>>
) -> HttpResponse {
    let token = if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                auth_str.trim_start_matches("Bearer ").to_string()
            } else {
                return HttpResponse::Unauthorized().finish();
            }
        } else {
            return HttpResponse::Unauthorized().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    };

    let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(secret_key.as_ref()),
        &jsonwebtoken::Validation::default()
    );

    let user_id = match token_data {
        Ok(data) => data.claims.sub,
        Err(err) => {
            error!("Error al decodificar JWT: {:?}", err);
            return HttpResponse::Unauthorized().finish();
        }
    };

    let ws = MyWs {
        state: data.get_ref().clone(),
        user_id,
        access: None,
    };

    ws::start(ws, &req, stream).expect("Error al iniciar WebSocket")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let client_id = ClientId::new(env::var("GITHUB_CLIENT_ID").expect("Falta el client id"));
    let client_secret = ClientSecret::new(env::var("GITHUB_CLIENT_SECRET").expect("Falta el client secret"));

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("URL de autorización inválida");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("URL de token inválida");

    let redirect_uri = RedirectUrl::new("http://localhost:8080/auth/callback".to_string())
        .expect("URL de redirección inválida");

    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_uri);

    let oauth_data = web::Data::new(OAuthData { client });

    info!("Iniciando servidor Actix-Web");

    let app_state = Arc::new(AppState {
        manager: Mutex::new(ProjectManager::new()),
    });

    #[get("/health")]
    async fn health() -> impl Responder {
        HttpResponse::Ok()
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(oauth_data.clone())
            .service(health)
            .service(auth)
            .service(auth_callback)
            .service(guest_jwt)
            .service(
                web::scope("")
                    .wrap(JwtMiddleware)
                    .service(websocket_handler)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}