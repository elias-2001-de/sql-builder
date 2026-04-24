use sql_builder::*;

// ── Schema ────────────────────────────────────────────────────────────────────

table! {
    users: Users => "users" {
        UserId*:    i64,
        UserName:   String,
        Email:      String,
        Age:        i32,
        Bio?:       String,
        AvatarUrl?: String
    }
}
impl_has_pk!(Users, users::UserId);

table! {
    posts: Posts => "posts" {
        PostId*:      i64,
        Title:        String,
        Body:         String,
        AuthorId->    Users: i64,
        PublishedAt?: i64,
        DeletedAt?:   i64
    }
}
impl_has_pk!(Posts, posts::PostId);

table! {
    comments: Comments => "comments" {
        CommentId*:  i64,
        Content:     String,
        PostId->     Posts: i64,
        AuthorId->   Users: i64,
        ParentId?->  Comments: i64
    }
}
impl_has_pk!(Comments, comments::CommentId);

// ── Demo ──────────────────────────────────────────────────────────────────────

fn main() {


    // ✅ PK lookup — typed_eq carries the column's value type
    let q1 = select()
        .from::<Posts>()
        .select_all()
        .where_col(typed_eq::<Posts, posts::PostId>(42_i64))
        .build();
    println!("Q1 (pk lookup):\n  {q1}\n");

    // ✅ JOIN via FK — posts::AuthorId: ForeignKey<Posts, References = Users>
    let q2 = select()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .where_col(gt::<Posts, posts::PostId>("100"))
        .order_by::<posts::PostId>(Direction::Desc)
        .limit(5)
        .build();
    println!("Q2 (join posts→users):\n  {q2}\n");

    // ✅ Multiple JOINs — comments has three FKs
    let q3 = select()
        .from::<Comments>()
        .select_all()
        .join::<Posts, comments::PostId>()
        .join::<Users, comments::AuthorId>()
        .left_join::<Comments, comments::ParentId>()
        .build();
    println!("Q3 (multi-join with self-ref):\n  {q3}\n");

    // ✅ Nullable FK — find top-level comments (no parent)
    let q4 = select()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Content)>()
        .where_col(is_null::<Comments, comments::ParentId>())
        .build();
    println!("Q4 (top-level comments):\n  {q4}\n");
}
