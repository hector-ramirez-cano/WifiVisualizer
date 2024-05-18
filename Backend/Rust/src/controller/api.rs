
use rocket::{data::N, fs::{relative, NamedFile}, http::{Cookie, CookieJar, SameSite}, response::Redirect, serde::json::Value};
use rocket_oauth2::{OAuth2, TokenResponse};

use crate::model::{db, types};

#[derive(Debug)]
pub struct Google;
pub struct GitHub;

const OAUTH2_TOKEN_COOKIE : & 'static str = "oauth_token";
const OAUTH2_USER_ID      : & 'static str = "oauth_user_id";

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

#[get("/login/google")]
pub fn google_login(oauth2: OAuth2<Google>, cookies: &CookieJar<'_>) -> Redirect {
    println!("[DEBUG]Logged in!");
    if cookies.get(OAUTH2_TOKEN_COOKIE).is_some() {

        println!("[DEBUG]Redirecting to /");
        Redirect::to("/")
    } else {

        // Redirect to google's auth server or local invalid login
        println!("[DEBUG]Redirecting to google auth api endpoint");
        oauth2.get_redirect(
            cookies,
            &[
                "https://www.googleapis.com/auth/userinfo.email",
                "openid",
                "https://www.googleapis.com/auth/userinfo.profile"
            ]).or_else(|_| Ok::<Redirect, rocket::Error>(Redirect::to("/invalid/login"))).unwrap()
    }
    
}

#[get("/auth/google")]
pub fn google_auth_callback(token: TokenResponse<Google>, cookies: &CookieJar<'_>) -> Redirect {
    // Set a private cookie with the access token
    cookies.add(
        Cookie::build((OAUTH2_TOKEN_COOKIE, token.access_token().to_string()))
            .same_site(SameSite::Lax)
            .http_only(false)
            .build()
    );
    
    Redirect::to("/")
}

#[get("/api/<user_id>/project_list")]
pub async fn api_get_project_list(user_id: &str, cookies : &CookieJar<'_>) -> Value {

    let cookie_id = cookies.get(&OAUTH2_USER_ID);
    let invalid = if let Some(val) = cookie_id {
        // name=value
        dbg!(&val.to_string().split("=").collect::<Vec<_>>());
        val.to_string().split("=").collect::<Vec<_>>()[1] != user_id
    } else { 
        true //cookie was none
    };

    if invalid {
        return rocket::serde::json::json!(
            {
                "code": 403,
                "comment": "Provided user id does not match logged in user id",
                "list": []
            }
        )
    }

    
    let internal_user = db::get_or_attempt_insert_user_id(user_id, "google").await;
    let internal_user = match internal_user {
        None => {
            return rocket::serde::json::json!(
                {
                    "code": 500,
                    "comment": "Could not create new user id",
                    "list": []
                }
            );
        }

        Some(id) => id
    };

    dbg!(&internal_user);
    

    // TODO: Consult db
    rocket::serde::json::json!(
        {
            "code": 200,
            "list": [
                {
                    "name": "Capture #1",
                    "description": "Wassup baby",
                    "status": "done",
                    "date": "01/02/03"
                },
                {
                    "name": "Capture #2",
                    "description": "How you livin'",
                    "status": "done",
                    "date": "01/02/03"
                },
                {
                    "name": "Capture #2",
                    "description": "Oh, you haven't seen how mean this dean can be- ean",
                    "status": "done",
                    "date": "01/02/03"
                },
                {
                    "name": "Noice",
                    "description": "I forgot everything you said after rectum!",
                    "status": "in capture",
                    "date": "01/02/03"
                }
            ]
        }
    )
}