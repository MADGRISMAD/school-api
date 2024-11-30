#[macro_use]
extern crate rocket;

use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{ClientOptions, FindOptions, InsertOneOptions, UpdateOptions},
    Client,
};
use futures::stream::TryStreamExt;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_cors::{CorsOptions};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct Student {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    age: u8,
    subject: String,
}

type Db = Arc<Client>;

/// Inicializa conexión a MongoDB
async fn init_db() -> Db {
    let client_uri = "mongodb://localhost:27017";
    let client_options = ClientOptions::parse(client_uri).await.unwrap();
    Arc::new(Client::with_options(client_options).unwrap())
}

/// Obtener todos los estudiantes
#[get("/students")]
async fn get_students(db: &rocket::State<Db>) -> Json<Vec<Student>> {
    let collection = db.database("school").collection::<Student>("students");
    let cursor = collection.find(None, FindOptions::builder().build()).await.unwrap();
    let students: Vec<Student> = cursor.try_collect().await.unwrap();
    Json(students)
}

/// Crear un nuevo estudiante
#[post("/students", format = "json", data = "<student>")]
async fn create_student(db: &rocket::State<Db>, student: Json<Student>) -> &'static str {
    let collection = db.database("school").collection::<Student>("students");
    collection.insert_one(student.into_inner(), InsertOneOptions::builder().build()).await.unwrap();
    "Student added successfully"
}

/// Obtener un estudiante por ID
#[get("/students/<id>")]
async fn get_student_by_id(db: &rocket::State<Db>, id: String) -> Option<Json<Student>> {
    let collection = db.database("school").collection::<Student>("students");
    let object_id = ObjectId::parse_str(&id).ok()?;
    collection.find_one(doc! { "_id": object_id }, None).await.unwrap().map(Json)
}

/// Actualizar un estudiante por ID
#[put("/students/<id>", format = "json", data = "<student>")]
async fn update_student(db: &rocket::State<Db>, id: String, student: Json<Student>) -> &'static str {
    let collection = db.database("school").collection::<Student>("students");
    let object_id = ObjectId::parse_str(&id).unwrap();
    collection.update_one(
        doc! { "_id": object_id },
        doc! { "$set": bson::to_document(&student.into_inner()).unwrap() },
        UpdateOptions::builder().build(),
    ).await.unwrap();
    "Student updated successfully"
}

/// Eliminar un estudiante por ID
#[delete("/students/<id>")]
async fn delete_student(db: &rocket::State<Db>, id: String) -> &'static str {
    let collection = db.database("school").collection::<Student>("students");
    let object_id = ObjectId::parse_str(&id).unwrap();
    collection.delete_one(doc! { "_id": object_id }, None).await.unwrap();
    "Student deleted successfully"
}

#[launch]
async fn rocket() -> _ {
    let db = init_db().await;

    // Configuración de CORS
    let cors = CorsOptions::default()
        .to_cors()
        .expect("Error al construir las opciones de CORS");

    rocket::build()
        .manage(db)
        .attach(cors) // Adjunta las reglas de CORS
        .mount("/", routes![get_students, create_student, get_student_by_id, update_student, delete_student])
}
