use actix_web::error::{self};
use actix_web::{App, HttpServer, Responder, put, web};

use chrono::{DateTime, Utc};
use dotenv::dotenv;
use rust_web_play::error::Error;
use serde::{Deserialize, Serialize};
use std::env;
use surrealdb::Connection;
use surrealdb::Surreal;
use surrealdb::engine::any;
use surrealdb::opt::auth::Root;

use rust_web_play::Res;

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    description: String,
    status: String,
    priority: String,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
}
pub async fn connect() -> Res<Surreal<any::Any>> {
    // Open a connection
    // let db: Surreal<Client> = Surreal::init();

    let url = env::var("SURREAL_URL").unwrap();
    println!("db url: {url}");
    // db.connect::<Wss>(url).await?;
    let db = any::connect(url).await?;
    // let jwt = env::var("SURREAL_AUTH_TOKEN").unwrap();
    // db.authenticate(jwt).await?;

    // Select namespace and database
    db.use_ns("demo").use_db("surreal_deal_store").await?;

    // Authenticate
    db.signin(Root {
        username: &env::var("SURREAL_USERNAME").unwrap(),
        password: &env::var("SURREAL_PASSWORD").unwrap(),
    })
    .await?;

    return Ok(db);
}

struct AppState {
    db: Surreal<any::Any>,
}

async fn create_project(db: &Surreal<impl Connection>) -> Res<Project> {
    // Create a record
    let project = Project {
        name: "SurrealDB Dashboard".to_string(),
        description: "A modern admin interface for SurrealDB".to_string(),
        status: "in_progress".to_string(),
        priority: "high".to_string(),
        tags: vec![
            "typescript".to_string(),
            "react".to_string(),
            "database".to_string(),
        ],
        created_at: Utc::now(),
    };

    let p: Option<Project> = db.create("project").content(project).await?;

    println!("project {p:#?}");
    p.ok_or_else(|| Error::Db("fail".to_owned()))
}

#[put("/project")]
async fn put_project(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let p = create_project(&state.db)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(format!("project created {p:#?}"))
}

async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    let db = connect().await?;
    let state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(put_project)
            .service(
                // prefixes all resources and routes attached to it...
                web::scope("/app")
                    // ...so this handles requests for `GET /app/index.html`
                    .route("/index.html", web::get().to(index)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
