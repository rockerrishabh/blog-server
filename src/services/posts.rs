use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use diesel::{
    query_dsl::methods::{FilterDsl, FindDsl, SelectDsl},
    result::{DatabaseErrorKind, Error::DatabaseError},
    ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper,
};
use serde::Deserialize;
use validator::Validate;

use crate::db::{
    connection::AppState,
    models::{CreatePost, Post},
};

#[derive(Deserialize, Validate, Debug)]
pub struct CreatePostRequest {
    #[validate(length(
        min = 10,
        max = 80,
        message = "Title length must be between 10 and 80 characters"
    ))]
    pub title: String,
    #[validate(length(
        min = 6,
        max = 200000,
        message = "Body length must be between 6 and 200000 characters"
    ))]
    pub body: String,
}

#[post("/posts/create")]
async fn create_post(
    data: Data<AppState>,
    body: Json<CreatePostRequest>,
    req: HttpRequest,
) -> impl Responder {
    use crate::db::schema::posts::dsl::{posts, title};

    let create_post_data = body.into_inner();

    if let Err(validation_errors) = create_post_data.validate() {
        let create_post_error_messages: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                format!(
                    "Invalid {}: {}",
                    field,
                    errors
                        .into_iter()
                        .map(|e| e.message.clone().unwrap_or_else(|| "Invalid value".into()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": create_post_error_messages,
        }));
    }

    if let Some(authenticated_user_id) = req.extensions().get::<String>() {
        match data.pool.get() {
            Ok(mut conn) => {
                let posts_exists = posts
                    .filter(title.eq(&create_post_data.title))
                    .first::<Post>(&mut conn);

                match posts_exists {
                    Ok(_) => {
                        return HttpResponse::Conflict().json(serde_json::json!({
                            "error": "Title already exists",
                    
                        }));
                    }
                    Err(_) => {
                        let new_post = CreatePost::new(
                            create_post_data.title,
                            create_post_data.body,
                            authenticated_user_id.to_string(),
                        );

                        match diesel::insert_into(posts)
                            .values(new_post)
                            .returning(Post::as_returning())
                            .get_result::<Post>(&mut conn)
                        {
                            Ok(post) => {
                                return HttpResponse::Created().json(serde_json::json!({
                                    "success": format!("Post successfully created with id {}", post.id),
                                   
                                }));
                            }
                            Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                                return HttpResponse::Conflict().json(serde_json::json!({
                                    "error": "Title already exists",
                               
                                }));
                            }
                            Err(e) => {
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                        "error": format!("An error occurred while deleting the post. Error:- {}", e),
                                        
                                    }),
                                );
                            }
                        }
                    }
                }
            }
            Err(_) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "An error occurred while connecting to the database",
             
                }));
            }
        }
    } else {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid token",
     
        }));
    }
}

#[get("/posts")]
async fn get_posts(data: Data<AppState>) -> impl Responder {
    use crate::db::schema::posts::dsl::posts;

    match data.pool.get() {
        Ok(mut conn) => {
            let all_posts = posts.select(Post::as_select()).load::<Post>(&mut conn);

            match all_posts {
                Ok(all) => HttpResponse::Ok().json(all),
                Err(_) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "No posts found in the database",
           
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An error occurred while connecting to the database",
        })),
    }
}

#[get("/posts/{post_id}")]
async fn get_post(data: Data<AppState>, path: Path<String>) -> impl Responder {
    use crate::db::schema::posts::dsl::posts;

    let post_id = path.into_inner();

    match data.pool.get() {
        Ok(mut conn) => {
            let get_post = posts
                .find(&post_id)
                .select(Post::as_select())
                .first::<Post>(&mut conn)
                .optional();

            match get_post {
                Ok(Some(post)) => HttpResponse::Ok().json(post),
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Post with id {} not found", post_id)
                })),
                Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "An error occurred while retrieving the post"
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An error occurred while connecting to the database"
        })),
    }
}

#[put("/posts/{post_id}/update")]
async fn update_post(
    data: Data<AppState>,
    path: Path<String>,
    update_body: Json<CreatePostRequest>,
    req: HttpRequest,
) -> impl Responder {
    use crate::db::schema::posts::dsl::{body, id, posts, title};

    let post_id = path.into_inner();
    let update_post_data = update_body.into_inner();

    if let Err(validation_errors) = update_post_data.validate() {
        let update_post_error_messages: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                format!(
                    "Invalid {}: {}",
                    field,
                    errors
                        .into_iter()
                        .map(|e| e.message.clone().unwrap_or_else(|| "Invalid value".into()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": update_post_error_messages
        }));
    }

    if let Some(authenticated_user_id) = req.extensions().get::<String>() {
        match data.pool.get() {
            Ok(mut conn) => {
                let post_exists = posts.filter(id.eq(&post_id)).first::<Post>(&mut conn);

                match post_exists {
                    Ok(post) => {
                        if post.user_id == Some(authenticated_user_id.to_string()) {
                            let updated_post = diesel::update(posts.find(post_id))
                                .set((
                                    title.eq(update_post_data.title),
                                    body.eq(update_post_data.body),
                                ))
                                .returning(Post::as_returning())
                                .get_result::<Post>(&mut conn);

                            match updated_post {
                                Ok(post) => HttpResponse::Ok().json(serde_json::json!({
                                    "success": format!("Post successfully updated with id {}", post.id),
                                    
                                })),
                                Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                                    HttpResponse::Conflict().json(serde_json::json!({
                                        "error": "Title already exists",
                                   
                                    }))
                                }
                                Err(e) => {
                                    HttpResponse::InternalServerError().json(serde_json::json!({
                                        "error": format!("An error occurred while updating the post. Error:- {}", e),
                                        
                                    }))
                                }
                            }
                        } else {
                            HttpResponse::Unauthorized().json(serde_json::json!({
                                "error": "You do not have permission to update this post",
                             
                            }))
                        }
                    }
                    Err(_) => HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Post with id {} not found", post_id),

                    })),
                }
            }
            Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "An error occurred while connecting to the database",

            })),
        }
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid token"
        }))
    }
}

#[delete("/posts/{post_id}/delete")]
async fn delete_post(data: Data<AppState>, path: Path<String>, req: HttpRequest) -> impl Responder {
    use crate::db::schema::posts::dsl::{id, posts};

    let post_id = path.into_inner();

    if let Some(authenticated_user_id) = req.extensions().get::<String>() {
        match data.pool.get() {
            Ok(mut conn) => {
                let post_exists = posts.filter(id.eq(&post_id)).first::<Post>(&mut conn);

                match post_exists {
                    Ok(post) => {
                        if post.user_id == Some(authenticated_user_id.to_string()) {
                            let deleted_post =
                                diesel::delete(posts.find(&post_id)).execute(&mut conn);

                            match deleted_post {
                                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                                    "success": format!("Post successfully deleted with id {}", post_id)
                                })),
                                Err(e) => {
                                    HttpResponse::InternalServerError().json(serde_json::json!({
                                        "error": format!("An error occurred while deleting the post. Error:- {}", e)
                            
                                    }))
                                }
                            }
                        } else {
                            HttpResponse::Unauthorized().json(serde_json::json!({
                                "error": "You do not have permission to delete this post",
                       
                            }))
                        }
                    }
                    Err(_) => HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Post with id {} not found", post_id)
                    })),
                }
            }
            Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "An error occurred while connecting to the database"
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid token",
  
        }))
    }
}
