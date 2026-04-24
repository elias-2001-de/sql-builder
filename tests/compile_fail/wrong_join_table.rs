// Joining a table that the FK does not point to must fail.
// posts::AuthorId references Users, not Comments.
use sql_builder::*;

#[derive(Table)]
#[table_name = "users"]
pub struct Users {
    #[primary_key]
    pub id: i64,
}

#[derive(Table)]
#[table_name = "posts"]
pub struct Posts {
    #[primary_key]
    pub id: i64,
    #[foreign_key(Users)]
    pub author_id: i64,
}

#[derive(Table)]
#[table_name = "comments"]
pub struct Comments {
    #[primary_key]
    pub id: i64,
}

fn main() {
    let _ = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .join::<Comments, posts::AuthorId>();
}
