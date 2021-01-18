use rocket_contrib::json::Json;
use crate::data::{Quiz, QUIZ_COLLECTION_NAME};
use rocket::http::CookieJar;
use rocket::State;
use mongodb::Database;
use crate::config::Config;
use crate::error::{Problem, problems};
use crate::jwt::{UserRolesToken, auth_problem};
use crate::role::Role;
use uuid::Uuid;
use crate::route::parse_uuid;
use bson::{Document, from_bson, Bson, doc};
use bson::spec::BinarySubtype;

// pub static PART_COLLECTION_NAME: &'static str = "parts";
// pub static PARTICIPANT_COLLECTION_NAME: &'static str = "participants";
// pub static QUIZ_COLLECTION_NAME: &'static str = "quizzes";

#[post("/", format = "application/json", data = "<quiz>")]
pub async fn quiz_create<'a>(quiz: Json<Quiz>, auth: UserRolesToken, db: State<'_, Database>) -> Result<(), Problem> {
    if !auth.has_min_role(Role::Author) {
        return Err(auth_problem("Permission level too low."));
    }

    db.collection(QUIZ_COLLECTION_NAME)
        .insert_one(bson::to_document(&quiz.0)
                        .expect("Unable to serialize Quiz struct into BSON."),
                    None).await
        .map_err(|e| Problem::from(e))?;

    Ok(())
}

#[inline]
pub fn quiz_id_filter(id: Uuid) -> Document {
    doc! {
        "id": Bson::Binary(bson::Binary {
            subtype: BinarySubtype::Uuid,
            bytes: id.as_bytes().to_vec(),
        })
    }
}

#[post("/<id>")]
pub async fn quiz_info<'a>(id: String, db: State<'_, Database>) -> Result<Option<Json<Quiz>>, Problem> {
    let uuid = parse_uuid(&id)?;

    let quiz_document = db.collection(QUIZ_COLLECTION_NAME).find_one(
        quiz_id_filter(uuid),
        None,
    ).await.expect("Unable to query by id.");

    let quiz: Option<Quiz> = match quiz_document {
        Some(doc) => Some(
            from_bson(Bson::Document(doc))
                .map_err(|e| Problem::from(e))?
        ),
        None => None
    };

    Ok(quiz.map(|u| Json(u)))
}


#[delete("/<id>")]
pub async fn quiz_delete<'a>(id: String, auth: UserRolesToken, db: State<'_, Database>) -> Result<Option<String>, Problem> {
    if !auth.has_min_role(Role::Author) {
        return Err(auth_problem("Permission level too low."));
    }

    let uuid = parse_uuid(&id)?;

    let quiz_document = db.collection(QUIZ_COLLECTION_NAME).find_one(
        quiz_id_filter(uuid),
        None,
    ).await.expect("Unable to query by id.");

    let quiz: Quiz = match quiz_document {
        Some(doc) => from_bson(Bson::Document(doc))
                .map_err(|e| Problem::from(e))?,
        None => return Ok(None)
    };

    if !auth.has_min_role(Role::Admin) && quiz.author != auth.user {
        return Err(auth_problem("Quiz not owned by user."));
    }

    db.collection(QUIZ_COLLECTION_NAME).delete_one(
        quiz_id_filter(uuid),
        None,
    ).await
        .map_err(|e| Problem::from(e))?;

    Ok(Some(uuid.to_string()))
}
