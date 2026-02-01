#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub page_path: String,
}

#[derive(Debug, Clone)]
pub struct Work {
    pub title: String,
    pub page_path: String,
}

#[derive(Debug, Clone)]
pub struct WorkLink {
    pub zip_path: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkRecord {
    pub author: String,
    pub title: String,
    pub text: String,
    pub url: Option<String>,
}
