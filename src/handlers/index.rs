use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Html;

use crate::state::AppState;
use super::extract_session;

const LOGGED_IN_HTML: &str = r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Whoop App</title>
<style>
body { font-family: -apple-system, system-ui, sans-serif; max-width: 600px; margin: 4rem auto; padding: 0 1rem; color: #e0e0e0; background: #1a1a1a; }
h1 { font-size: 1.4rem; color: #4ade80; }
.status { background: #22c55e22; border: 1px solid #22c55e44; border-radius: 8px; padding: 1rem; margin: 1rem 0; }
a { color: #60a5fa; text-decoration: none; }
a:hover { text-decoration: underline; }
ul { list-style: none; padding: 0; }
li { padding: 0.4rem 0; }
.logout { color: #f87171; }
</style></head><body>
<h1>&#9989; Logged in to Whoop</h1>
<div class="status">Session active</div>
<h2 style="font-size:1.1rem;">API Endpoints</h2>
<ul>
<li><a href="/api/profile">/api/profile</a> &#8212; your profile</li>
<li><a href="/api/body">/api/body</a> &#8212; body measurements</li>
<li><a href="/api/cycles">/api/cycles</a> &#8212; physiological cycles</li>
<li><a href="/api/sleep">/api/sleep</a> &#8212; sleep records</li>
<li><a href="/api/workouts">/api/workouts</a> &#8212; workouts</li>
</ul>
<script>
function doLogout() { fetch("/auth/logout", {method: "DELETE"}).then(function() { location.reload(); }); }
</script>
<p><a class="logout" href="javascript:doLogout()">Log out</a></p>
</body></html>"##;

const LOGGED_OUT_HTML: &str = r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Whoop App</title>
<style>
body { font-family: -apple-system, system-ui, sans-serif; max-width: 600px; margin: 4rem auto; padding: 0 1rem; color: #e0e0e0; background: #1a1a1a; text-align: center; }
h1 { font-size: 1.4rem; }
a.login { display: inline-block; margin-top: 1rem; padding: 0.75rem 2rem; background: #2563eb; color: white; border-radius: 8px; text-decoration: none; font-weight: 600; }
a.login:hover { background: #1d4ed8; }
</style></head><body>
<h1>Whoop App</h1>
<p>Connect your Whoop account to get started.</p>
<a class="login" href="/auth/login">Log in with Whoop</a>
</body></html>"##;

pub async fn index(headers: HeaderMap, State(state): State<AppState>) -> Html<&'static str> {
    let session = extract_session(&headers).ok();
    let logged_in = if let Some(sid) = session {
        let tokens = state.tokens.read().await;
        tokens.contains_key(&sid)
    } else {
        false
    };

    if logged_in {
        Html(LOGGED_IN_HTML)
    } else {
        Html(LOGGED_OUT_HTML)
    }
}
