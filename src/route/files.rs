use std::path::PathBuf;

use rocket::State;
use rocket::response::NamedFile;

use crate::config::Config;

pub async fn app_index_file(c: State<'_, Config>) -> NamedFile {
    NamedFile::open(c.public_content.as_path().join("index.html"))
        .await
        .expect(
            format!("'{}' does not exist!",
                    c.public_content.as_path()
                        .join("index.html")
                        .display()
            ).as_str()
        )
}

#[get("/", format = "text/html")]
pub async fn app(c: State<'_, Config>) -> NamedFile {
    app_index_file(c).await
}

#[get("/<path..>", format = "text/html", rank = 10)]
pub async fn app_path(path: PathBuf, c: State<'_, Config>) -> NamedFile {
    NamedFile::open(c.public_content.as_path().join(path.as_path())).await
        .ok()
        .unwrap_or(app_index_file(c).await)
}
