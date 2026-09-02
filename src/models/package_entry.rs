#[derive(Debug, Clone, Default)]
pub struct PackageEntry {
    pub id: String,
    pub version: Option<String>,
}

impl PackageEntry {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: None,
        }
    }
}
