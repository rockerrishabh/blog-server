match posts_exists {
    Ok(Some(_)) => {
        HttpResponse::Conflict().json(CreatePostResponse::TitleAlreadyExists)
    }
    Ok(None) => {
       
    }
    Err(_) => {
        return HttpResponse::InternalServerError().json(
            CreatePostResponse::InternalError(
                "An error occured while connecting to the database".to_string(),
            ),
        )
    }
}
}