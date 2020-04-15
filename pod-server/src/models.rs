use crate::schema::users;

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub pub_key: String, // base64 encoded
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub login: String,
    pub pub_key: String, // base64 encoded
}
