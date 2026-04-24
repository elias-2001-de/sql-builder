// Joining a table that the FK does not point to must fail.
// posts::AuthorId references Users, not Comments.
use sql_builder::*;

table! { users: Users => "users" { UserId*: i64 } }
impl_has_pk!(Users, users::UserId);

table! {
    posts: Posts => "posts" {
        PostId*:   i64,
        AuthorId-> Users: i64
    }
}
impl_has_pk!(Posts, posts::PostId);

table! { comments: Comments => "comments" { CommentId*: i64 } }
impl_has_pk!(Comments, comments::CommentId);

fn main() {
    let _ = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .join::<Comments, posts::AuthorId>();
}
