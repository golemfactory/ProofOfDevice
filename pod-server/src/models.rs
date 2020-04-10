use crate::schema::users;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub pub_key: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub login: &'a str,
    pub pub_key: &'a str,
}