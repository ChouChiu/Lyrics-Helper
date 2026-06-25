use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent("Lyrics-Helper/0.1")
        .build()
        .expect("Failed to create HTTP client")
});

pub async fn get_json<T: DeserializeOwned>(url: &str) -> Option<T> {
    let response = HTTP_CLIENT.get(url).send().await.ok()?;
    response.json::<T>().await.ok()
}

pub async fn get_json_with_headers<T: DeserializeOwned>(
    url: &str,
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.get(url);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}

pub async fn post_json<T: DeserializeOwned>(url: &str, body: &impl Serialize) -> Option<T> {
    let response = HTTP_CLIENT.post(url).json(body).send().await.ok()?;
    response.json::<T>().await.ok()
}

pub async fn post_json_with_headers<T: DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.post(url).json(body);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}

pub async fn post_json_raw_with_headers(
    url: &str,
    body: &impl Serialize,
    headers: &[(&str, &str)],
) -> Option<String> {
    let mut request = HTTP_CLIENT.post(url).json(body);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.text().await.ok()
}

pub async fn post_form<T: DeserializeOwned>(
    url: &str,
    form: &[(&str, &str)],
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.post(url).form(form);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}
