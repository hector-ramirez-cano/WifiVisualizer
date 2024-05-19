use rocket::{http::{Cookie, CookieJar, SameSite}, response::Redirect, time::Duration};
use rocket_oauth2::{OAuth2, TokenResponse};

use crate::controller::api::OAUTH2_TOKEN_COOKIE;


#[derive(Debug)]
pub struct Google;
pub struct GitHub;

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
    let offset = token.expires_in().unwrap_or(0);
    let offset = Duration::seconds(offset);
    let offset = rocket::time::OffsetDateTime::now_utc().checked_add(offset);

    cookies.add(
        Cookie::build((OAUTH2_TOKEN_COOKIE, token.access_token().to_string()))
            .same_site(SameSite::Lax)
            .http_only(false)
            .expires(offset)
            .build()
    );
    
    Redirect::to("/")
}
