extern crate hex;

use self::hex::FromHex;
use slack::api::User as SlackUser;
use tui::style::Color;

use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserID(String);

#[derive(Debug, Clone)]
pub struct User {
    id: UserID,
    display_name: String,
    color: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Default)]
pub struct UserList {
    users: BTreeMap<UserID, User>,
}

type Iter<'a> = ::std::collections::btree_map::Iter<'a, UserID, User>;

impl User {
    pub fn from_slack(slack_user: &SlackUser) -> Option<User> {
        let id = match slack_user.id {
            Some(ref id) => UserID::from(id),
            None => return None,
        };

        let hex_color = slack_user.color.as_ref().map(String::as_ref).unwrap_or("");

        // TODO: slack_api 0.18 does not have everything that we need under the UserProfile key
        // let profile = slack_user.profile.as_ref();

        Some(User {
            id,
            color: <[u8; 3]>::from_hex(hex_color).ok(),
            display_name: slack_user
                .name
                .clone()
                .unwrap_or_else(|| String::from("No name")),
        })
    }

    #[cfg(test)]
    pub fn fixture<I, S>(id: I, display_name: S) -> User
    where
        I: Into<UserID>,
        S: Into<String>,
    {
        User {
            id: id.into(),
            color: None,
            display_name: display_name.into(),
        }
    }

    pub fn id(&self) -> &UserID {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn color(&self) -> Color {
        match self.color {
            Some(rgb) => Color::Rgb(rgb[0], rgb[1], rgb[2]),
            None => Color::Reset,
        }
    }
}

impl UserList {
    pub fn new() -> Self {
        UserList {
            users: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id().clone(), user);
    }

    pub fn iter(&self) -> Iter {
        self.users.iter()
    }

    pub fn get(&self, id: &UserID) -> Option<&User> {
        self.users.get(id)
    }
}

impl UserID {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a str> for UserID {
    fn from(s: &'a str) -> Self {
        UserID(String::from(s))
    }
}

impl<'a> From<&'a String> for UserID {
    fn from(s: &'a String) -> Self {
        UserID(s.clone())
    }
}

impl From<String> for UserID {
    fn from(s: String) -> Self {
        UserID(s)
    }
}

impl FromIterator<User> for UserList {
    fn from_iter<I: IntoIterator<Item = User>>(iter: I) -> Self {
        UserList {
            users: iter.into_iter().map(|c| (c.id.clone(), c)).collect(),
        }
    }
}
