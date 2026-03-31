use crate::blog::Post;
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate;

#[derive(Template)]
#[template(path = "home_fragment.html")]
pub struct HomeFragmentTemplate;

#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate;

#[derive(Template)]
#[template(path = "resume_fragment.html")]
pub struct ResumeFragmentTemplate;

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub status: u16,
    pub message: String,
}

#[derive(Template)]
#[template(path = "blog/list.html")]
pub struct BlogListTemplate {
    pub posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "blog/list_fragment.html")]
pub struct BlogListFragmentTemplate {
    pub posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "blog/post.html")]
pub struct BlogPostTemplate {
    pub post: Post,
}

#[derive(Template)]
#[template(path = "blog/post_fragment.html")]
pub struct BlogPostFragmentTemplate {
    pub post: Post,
}
