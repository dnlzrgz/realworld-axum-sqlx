use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::profiles::Profile;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub comment_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: String,
    pub author: Profile,
}

#[derive(Serialize, Deserialize)]
pub struct CommentBody<T = Comment> {
    pub comment: T,
}

#[derive(Serialize)]
pub struct MultipleCommentsBody {
    pub comments: Vec<Comment>,
}

#[derive(Deserialize)]
pub struct AddComment {
    pub body: String,
}

pub struct CommentFromQuery {
    pub comment_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: String,
    pub author_username: String,
    pub author_bio: String,
    pub author_image: Option<String>,
    pub following_author: bool,
}

impl CommentFromQuery {
    pub fn into_comment(self) -> Comment {
        Comment {
            comment_id: self.comment_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            body: self.body,
            author: Profile {
                username: self.author_username,
                bio: self.author_bio,
                image: self.author_image,
                following: self.following_author,
            },
        }
    }
}
