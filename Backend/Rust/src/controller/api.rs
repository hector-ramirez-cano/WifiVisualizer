
use rocket::{fs::{relative, NamedFile}, http::{Cookie, CookieJar, SameSite}, response::Redirect};
use rocket_oauth2::{OAuth2, TokenResponse};

#[derive(Debug)]
pub struct Google;
pub struct GitHub;

const OAUTH2_TOKEN_COOKIE : & 'static str = "oauth_token";


#[get("/")]
pub fn index() -> Redirect {
    Redirect::to("/home")
}

#[get("/home")]
pub async fn home() -> Option<NamedFile> {
    NamedFile::open(relative!("../../Frontend/public/home.html")).await.ok()
}

#[get("/login")]
pub async fn login() -> Option<NamedFile> {
    NamedFile::open(relative!("../../Frontend/public/login.html")).await.ok()
}

#[get("/view")]
pub async fn view() -> Option<NamedFile> {
    NamedFile::open(relative!("../../Frontend/public/view.html")).await.ok()
}

#[get("/invalid/<msg>")]
pub fn invalid_msg(msg: &str) -> String {
    let mut banner = "Unexpected error ocurred: ".to_string();
    banner.push_str(msg);

    banner
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(OAUTH2_TOKEN_COOKIE);

    Redirect::to("/")
}

#[get("/login/google")]
pub fn google_login(oauth2: OAuth2<Google>, cookies: &CookieJar<'_>) -> Redirect {
    if let Some(_token) = cookies.get(OAUTH2_TOKEN_COOKIE) {

        Redirect::to("/")
    } else {

        // Redirect to google's auth server or local invalid login
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
    dbg!(&cookies);
    dbg!(&token);
    cookies.add_private(
        Cookie::build((OAUTH2_TOKEN_COOKIE, token.access_token().to_string()))
            .same_site(SameSite::Lax)
            .build()
    );
    Redirect::to("/")
}
