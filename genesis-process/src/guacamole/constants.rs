// 字面量常量
pub const GUA_USERNAME: &str = "username";
pub const GUA_PASSWORD: &str = "password";
pub const GUA_WIDTH: &str = "width";
pub const GUA_HEIGHT: &str = "height";
pub const GUA_DPI: &str = "dpi";
pub const GUA_HOSTNAME: &str = "hostname";
pub const GUA_HOST_PORT: &str = "port";
pub const GUA_TRUE: &str = "true";
pub const GUA_FALSE: &str = "false";
pub const GUA_EMPTY: &str = "";

// RDP 参数常量
pub const RDP_CLIENT_NAME: &str = "client-name";
pub const RDP_DRIVE_PATH: &str = "drive-path";
pub const RDP_DRIVE_NAME: &str = "drive-name";
pub const RDP_ENABLE_DRIVE: &str = "enable-drive";
pub const RDP_DISABLE_UPLOAD: &str = "disable-upload";
pub const RDP_DISABLE_DOWNLOAD: &str = "disable-download";
pub const RDP_SECURITY: &str = "security";
pub const RDP_IGNORE_CERT: &str = "ignore-cert";
pub const RDP_CREATE_DRIVE_PATH: &str = "create-drive-path";
pub const RDP_RESIZE_METHOD: &str = "resize-method";

pub const ENABLE_RECORDING: &str = "enable-recording";
pub const RECORDING_PATH: &str = "recording-path";
pub const RECORDING_NAME: &str = "recording-name";
pub const CREATE_RECORDING_PATH: &str = "create-recording-path";

pub const FONT_NAME: &str = "font-name";
pub const FONT_SIZE: &str = "font-size";
pub const COLOR_SCHEME: &str = "color-scheme";
pub const BACKSPACE: &str = "backspace";
pub const TERMINAL_TYPE: &str = "terminal-type";

pub const PRE_CONNECTION_ID: &str = "preconnection-id";
pub const PRE_CONNECTION_BLOB: &str = "preconnection-blob";

pub const ENABLE_WALLPAPER: &str = "enable-wallpaper";
pub const ENABLE_THEMING: &str = "enable-theming";
pub const ENABLE_FONT_SMOOTHING: &str = "enable-font-smoothing";
pub const ENABLE_FULL_WINDOW_DRAG: &str = "enable-full-window-drag";
pub const ENABLE_DESKTOP_COMPOSITION: &str = "enable-desktop-composition";
pub const ENABLE_MENU_ANIMATIONS: &str = "enable-menu-animations";
pub const DISABLE_BITMAP_CACHING: &str = "disable-bitmap-caching";
pub const DISABLE_OFFSCREEN_CACHING: &str = "disable-offscreen-caching";
pub const FORCE_LOSSLESS: &str = "force-lossless";

pub const DOMAIN: &str = "domain";
pub const REMOTE_APP: &str = "remote-app";
pub const REMOTE_APP_DIR: &str = "remote-app-dir";
pub const REMOTE_APP_ARGS: &str = "remote-app-args";

pub const COLOR_DEPTH: &str = "color-depth";
pub const CURSOR: &str = "cursor";
pub const SWAP_RED_BLUE: &str = "swap-red-blue";
pub const DEST_HOST: &str = "dest-host";
pub const DEST_PORT: &str = "dest-port";
pub const READ_ONLY: &str = "read-only";

pub const USERNAME_REGEX: &str = "username-regex";
pub const PASSWORD_REGEX: &str = "password-regex";
pub const LOGIN_SUCCESS_REGEX: &str = "login-success-regex";
pub const LOGIN_FAILURE_REGEX: &str = "login-failure-regex";

pub const DELIMITER: char = ';';
pub const VERSION: &str = "VERSION_1_4_0";
pub const TUNNEL_CLOSED: i32 = -1;

pub const RDP_PARAMETER_NAMES: &[&str] = &[
    DOMAIN,
    REMOTE_APP,
    REMOTE_APP_DIR,
    REMOTE_APP_ARGS,
    RDP_ENABLE_DRIVE,
    RDP_DRIVE_PATH,
    FORCE_LOSSLESS,
    ENABLE_WALLPAPER,
    ENABLE_THEMING,
    ENABLE_FONT_SMOOTHING,
    ENABLE_FULL_WINDOW_DRAG,
    ENABLE_DESKTOP_COMPOSITION,
    ENABLE_MENU_ANIMATIONS,
    DISABLE_BITMAP_CACHING,
    DISABLE_OFFSCREEN_CACHING,
    COLOR_DEPTH,
    PRE_CONNECTION_ID,
    PRE_CONNECTION_BLOB,
    RDP_SECURITY,
    RDP_IGNORE_CERT,
    RDP_CREATE_DRIVE_PATH,
    RDP_RESIZE_METHOD,
];

pub const VNC_PARAMETER_NAMES: &[&str] =
    &[COLOR_DEPTH, CURSOR, SWAP_RED_BLUE, DEST_HOST, DEST_PORT];

pub const TELNET_PARAMETER_NAMES: &[&str] = &[
    FONT_NAME,
    FONT_SIZE,
    COLOR_SCHEME,
    BACKSPACE,
    TERMINAL_TYPE,
    USERNAME_REGEX,
    PASSWORD_REGEX,
    LOGIN_SUCCESS_REGEX,
    LOGIN_FAILURE_REGEX,
];

use std::collections::HashMap;

pub fn default_rdp_properties() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (ENABLE_RECORDING, "true"),
        (FONT_NAME, "menlo"),
        (FONT_SIZE, "12"),
        (COLOR_SCHEME, "gray-black"),
        (ENABLE_WALLPAPER, "true"),
        (ENABLE_THEMING, "true"),
        (ENABLE_FONT_SMOOTHING, "true"),
        (ENABLE_FULL_WINDOW_DRAG, "true"),
        (ENABLE_DESKTOP_COMPOSITION, "true"),
        (ENABLE_MENU_ANIMATIONS, "true"),
        (DISABLE_BITMAP_CACHING, "false"),
        (DISABLE_OFFSCREEN_CACHING, "false"),
        (RDP_SECURITY, "any"),
        (RDP_IGNORE_CERT, "true"),
        (RDP_CREATE_DRIVE_PATH, "true"),
        (RDP_RESIZE_METHOD, "display-update"),
    ])
}
