#[inline]
pub const fn _default_ssh_port() -> u16 {
    22
}

#[inline]
pub fn _default_username() -> String {
    "root".to_owned()
}

#[inline]
pub fn _default_recording_path() -> String {
    "./".to_owned()
}
