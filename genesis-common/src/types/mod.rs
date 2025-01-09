mod aliases;

pub use aliases::*;

pub enum NotifyEnum {
    INIT,
    SUCCESS,
    ERROR(String),
}
