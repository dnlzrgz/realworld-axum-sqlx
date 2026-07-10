use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::profiles::Profile;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: i64,
    pub author: Profile,
}

#[derive(Deserialize, Serialize)]
pub struct ArticleBody<T = Article> {
    pub article: T,
}

#[derive(Serialize)]
pub struct TagsBody {
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub tag_list: Option<Vec<String>>,
}

pub struct ArticleFromQuery {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorited: bool,
    pub favorites_count: i64,
    pub author_username: String,
    pub author_bio: String,
    pub author_image: Option<String>,
    pub following_author: bool,
}

impl ArticleFromQuery {
    pub fn into_article(self) -> Article {
        Article {
            slug: self.slug,
            title: self.title,
            description: self.description,
            body: self.body,
            tag_list: self.tag_list,
            created_at: self.created_at,
            updated_at: self.updated_at,
            favorited: self.favorited,
            favorites_count: self.favorites_count,
            author: Profile {
                username: self.author_username,
                bio: self.author_bio,
                image: self.author_image,
                following: self.following_author,
            },
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ListArticlesQuery {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct FeedArticlesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleArticlesBody {
    pub articles: Vec<Article>,
    pub articles_count: usize,
}
