use sea_orm::Set;
use uuid::Uuid;
use entity::{roles, users};

pub struct UserFactory {
    pub id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password: String,
}

impl UserFactory {
    pub fn new() -> UserFactory {
        UserFactory {
            id: Some(Uuid::new_v4()),
            name: String::from("John Doe"),
            username: String::from("johndoe"),
            email: String::from("johndoe@example.com"),
            password: String::from("password")
        }
    }

    pub fn id(mut self, id: Uuid) -> UserFactory {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: String) -> UserFactory {
        self.name = name;
        self
    }

    pub fn username(mut self, username: String) -> UserFactory {
        self.username = username;
        self
    }

    pub fn email(mut self, email: String) -> UserFactory {
        self.email = email;
        self
    }

    pub fn password(mut self, password: String) -> UserFactory {
        self.password = password;
        self
    }

    pub fn build(self) -> users::ActiveModel {
        users::ActiveModel {
            id: Set(self.id.unwrap_or(Uuid::new_v4())),
            name: Set(self.name),
            username: Set(self.username),
            email: Set(self.email),
            password: Set(self.password),
            ..Default::default()
        }
    }
}

pub struct RoleFactory {
    name: String,
}

impl RoleFactory {
    pub fn new() -> RoleFactory {
        RoleFactory {
            name: String::from("admin"),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> RoleFactory {
        self.name = name.into();
        self
    }

    pub fn build(self) -> roles::ActiveModel {
        roles::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(self.name),
            ..Default::default()
        }
    }
}