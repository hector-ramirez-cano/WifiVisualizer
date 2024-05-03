
use rocket::{fs::NamedFile, response::Redirect, fs::relative};

use std::path::{Path, PathBuf};


#[get("/")]
pub fn index() -> &'static str {
    "Hello, World!"
}

#[get("/login")]
pub async fn login() -> Option<NamedFile> {
    NamedFile::open(relative!("../../../Frontend/public/login.html")).await.ok()
}

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("public/").join(file);
    dbg!(&path);
    NamedFile::open(path).await.ok()
}