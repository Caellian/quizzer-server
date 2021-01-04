use num_enum::IntoPrimitive;

#[derive(Debug, Clone, Copy, IntoPrimitive, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Role {
    Normal,
    Author,
    Admin,
}

impl Role {
    /// Indicates whether user with role can create Quizzes
    pub fn can_author(self) -> bool {
        self >= Role::Author
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Normal => write!(f, "normal"),
            Role::Author => write!(f, "author"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

impl std::convert::Into<String> for Role {
    fn into(self) -> String {
        self.to_string()
    }
}
