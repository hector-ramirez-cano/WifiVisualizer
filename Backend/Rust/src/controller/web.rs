use rocket::{fs::{relative, NamedFile}, http::CookieJar, response::Redirect};

use crate::controller::api::{OAUTH2_USER_ID, OAUTH2_TOKEN_COOKIE};


macro_rules! login_guard {
    ( $cookies:expr ) => {
        if $cookies.get(&OAUTH2_TOKEN_COOKIE).is_none() {
            println!("[DEBUG]Error! Not logged in!");
            return Err(Redirect::to("/login"));
        }    
    };
}

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to("/home")
}

#[get("/home")]
pub async fn home(cookies: &CookieJar<'_>) -> Result<NamedFile, Redirect> {
    login_guard!(cookies);

    NamedFile::open(relative!("../../Frontend/public/nav.html")).await.or_else(|_| Err(Redirect::to("/error")))
}
#[get("/view")]
pub async fn view(cookies: &CookieJar<'_>) -> Result<NamedFile, Redirect> {
    login_guard!(cookies);

    NamedFile::open(relative!("../../Frontend/public/nav.html")).await.or_else(|_| Err(Redirect::to("/error")))
}

#[get("/capture")]
pub async fn capture(cookies: &CookieJar<'_>) -> Result<NamedFile, Redirect> {
    login_guard!(cookies);

    NamedFile::open(relative!("../../Frontend/public/nav.html")).await.or_else(|_| Err(Redirect::to("/error")))
}

#[get("/login")]
pub async fn login() -> Option<NamedFile> {
    NamedFile::open(relative!("../../Frontend/public/login.html")).await.ok()
}



#[get("/invalid/<msg>")]
pub fn invalid_msg(msg: &str) -> String {
    let mut banner = "Unexpected error ocurred: ".to_string();
    banner.push_str(msg);

    banner
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(OAUTH2_TOKEN_COOKIE);
    cookies.remove(OAUTH2_USER_ID);
    

    Redirect::to("/")
}

