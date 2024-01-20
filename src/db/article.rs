
use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize,Debug)]
pub struct ArticleLinkContent{
    pub content_id: i32,
    pub draft_content_id: i32,
    pub summary_content_id: i32,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct Article{
    pub id: i32,
    pub author: String,
    #[serde(flatten)]
    pub link_content: Option<ArticleLinkContent>,
    pub template_id: i32,
    pub cover_id: i32,
    pub visits: i32,
    pub coments: i32,
    pub public_state: i16,
    pub draft_state: i16,
    pub is_pinned: i16,
    pub is_commentable: i16,
    // pub created_at: DateTime,
    // pub updated_at: DateTime,
    pub password: Option<String>,
    pub title: Option<String>,
    pub alias: Option<String>,
}