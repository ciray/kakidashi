#[derive(Debug, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub page_path: String,
}

#[derive(Debug, Clone)]
pub struct Work {
    pub id: String,
    pub title: String,
    pub page_path: String,
}

#[derive(Debug, Clone)]
pub struct WorkRecord {
    pub author_id: String,
    pub author_name: String,
    pub work_id: String,
    pub work_title: String,
    pub zip_file_path: String,
}
