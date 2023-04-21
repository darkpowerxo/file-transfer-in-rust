#[macro_use] extern crate rocket;

use rocket::Request;
use rocket::fs::{TempFile, NamedFile};
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::FromRequest;
use rocket::response::status::NotFound; 
use uuid:: Uuid;
use std::collections::HashMap;
use std::path::Path; 
use std::fs;
struct RequestHeaders { 
    content_type: String, 
}
#[derive (Debug)]
enum RequestHeadersError { 
    BadThingsHappened,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders {
    type Error = RequestHeadersError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut headers = HashMap::<String,String>::new();
        for header in request.headers().iter() {
            headers.insert(header.name().to_owned().to_string(),header.value().to_owned().to_string());
        }
        let must_have = vec!["content-type"];
        for item in must_have {
            if !headers.contains_key(item){
                return Outcome::Failure((Status::BadRequest,RequestHeadersError::BadThingsHappened));
            }
        }
        let return_headers = RequestHeaders {
            content_type: headers.get("content-type").unwrap().to_owned(),
        };
        return Outcome::Success(return_headers)
    }
}

#[post("/upload", data = "<file>")]
async fn upload(mut file: TempFile<'_>, headers: RequestHeaders) -> std::io::Result<String> { 
    let id = Uuid::new_v4();

    let content_type = headers.content_type;

    let extension = mime_guess::get_mime_extensions_str(&content_type).unwrap();

    let form = format!("./files/{}.{}", id.to_string(), extension[0]); 
    let path = Path::new(&form);

    file.persist_to(path).await?;
    Ok(format!("{}.{}", id.to_string(), extension[0]))
}

#[get("/download/<identifier>")]
async fn download (identifier: &str)-> Result<NamedFile, NotFound<String>> {
    let form= format! ("./files/{}", identifier);
    let path = Path:: new(&form);
    if path.exists() {
        Err(NotFound("no file!".to_string()))
    } else {
        let res = NamedFile:: open(&path).await.map_err(|e| NotFound (e. to_string())); 
        match res {
            Ok (file) => Ok(file),
            Err(error) => panic! ("Problem with file {:?}", error),
        }
    }
}

#[delete("/delete/<identifier>")]
fn delete(identifier: &str) -> std::io::Result<String> {
    let form = format! ("./files/{}", identifier);
    if !Path:: new(&form).exists() {
        Ok("no file!".to_string())
    } else {
        fs:: remove_file(&form)?;
        Ok("deleted!".to_string())
    }
}

#[put("/replace/<identifier>", data = "<file>")]
async fn replace(identifier: &str, mut file: TempFile<'_>) -> std::io::Result<String> {
    let form = format! ("./files/{}", identifier);
    if !Path::new(&form).exists() {
        Ok("no file!".to_string())
    } else {
        let path = Path:: new(&form);
        file.persist_to(path).await?;
        Ok("replaced!".to_string())
    }
}

#[get("/list")]
fn list()-> String {
    let paths = fs::read_dir("./files/").unwrap();

    let mut all_paths = "".to_string();

    for path in paths {
        let mut path = path.unwrap().path().display().to_string() + "\n";
        path = path[8..].to_string();
        if path.chars().nth(0).unwrap() == '.' {
            continue
        } else {
            all_paths.push_str(&path);
        }
    }

    let mut chars = all_paths.chars(); 

    chars.next_back();
    chars.as_str().to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/v1", routes![
        upload,
        download,
        delete,
        replace,
        list
        ])
}