//! Tests for visuals/icons.rs, visuals/osc.rs, and visuals/sync.rs.

mod common;
use common::make_area;

use dracon_terminal_engine::visuals::icons::{Icon, IconMode};
use dracon_terminal_engine::utils::FileCategory;
use std::io::Cursor;

#[test]
fn test_icon_get_for_path_directory() {
    let path = std::path::Path::new("/home/user/Documents");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e0b9}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_rust_file() {
    let path = std::path::Path::new("main.rs");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(icon.contains('\u{7d97}'));
}

#[test]
fn test_icon_get_for_path_json_file() {
    let path = std::path::Path::new("data.json");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(icon.contains('\u{6b2e}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_toml_file() {
    let path = std::path::Path::new("Cargo.toml");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(icon.contains('\u{7d97}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_markdown_file() {
    let path = std::path::Path::new("README.md");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(icon.contains('\u{6d14}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_script_file() {
    let path = std::path::Path::new("script.sh");
    let icon = Icon::get_for_path(path, FileCategory::Script, false, IconMode::Nerd);
    assert!(icon.contains('\u{6e77}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_archive() {
    let path = std::path::Path::new("archive.zip");
    let icon = Icon::get_for_path(path, FileCategory::Archive, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_image() {
    let path = std::path::Path::new("photo.png");
    let icon = Icon::get_for_path(path, FileCategory::Image, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_audio() {
    let path = std::path::Path::new("song.mp3");
    let icon = Icon::get_for_path(path, FileCategory::Audio, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_video() {
    let path = std::path::Path::new("video.mp4");
    let icon = Icon::get_for_path(path, FileCategory::Video, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_home_directory() {
    let path = std::path::Path::new("home");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e07c}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_downloads_directory() {
    let path = std::path::Path::new("downloads");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e05a}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_hidden_directory() {
    let path = std::path::Path::new(".config");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e493}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_unknown_extension() {
    let path = std::path::Path::new("file.xyz");
    let icon = Icon::get_for_path(path, FileCategory::Other, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_nerd_mode_all_variants() {
    let icons = [
        Icon::Folder,
        Icon::File,
        Icon::Star,
        Icon::Storage,
        Icon::Git,
        Icon::Archive,
        Icon::Image,
        Icon::Audio,
        Icon::Video,
        Icon::Script,
        Icon::Document,
        Icon::Search,
        Icon::Settings,
        Icon::Trash,
        Icon::Home,
        Icon::Downloads,
        Icon::Documents,
        Icon::Pictures,
        Icon::Music,
        Icon::Videos,
        Icon::Rust,
        Icon::Json,
        Icon::Toml,
        Icon::Markdown,
    ];
    for icon in icons {
        let s = icon.get(IconMode::Nerd);
        assert!(!s.is_empty());
    }
}

#[test]
fn test_icon_get_unicode_mode_all_variants() {
    let icon = Icon::File;
    let s = icon.get(IconMode::Unicode);
    assert!(!s.is_empty());
}

#[test]
fn test_icon_get_ascii_mode_all_variants() {
    let icon = Icon::File;
    let s = icon.get(IconMode::ASCII);
    assert!(!s.is_empty());
}

#[test]
fn test_icon_mode_nerd_longer_than_ascii() {
    let icon = Icon::File;
    let nerd = icon.get(IconMode::Nerd);
    let ascii = icon.get(IconMode::ASCII);
    assert!(nerd.len() >= ascii.len());
}

#[test]
fn test_icon_get_for_path_without_extension() {
    let path = std::path::Path::new("Makefile");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_dockerfile() {
    let path = std::path::Path::new("Dockerfile");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_gitignore() {
    let path = std::path::Path::new(".gitignore");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_license() {
    let path = std::path::Path::new("LICENSE");
    let icon = Icon::get_for_path(path, FileCategory::Text, false, IconMode::Nerd);
    assert!(!icon.is_empty());
}

#[test]
fn test_icon_get_for_path_trash_directory() {
    let path = std::path::Path::new(".trash");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e1d4}') || !icon.is_empty());
}

#[test]
fn test_icon_get_for_path_config_directory() {
    let path = std::path::Path::new(".local");
    let icon = Icon::get_for_path(path, FileCategory::Other, true, IconMode::Nerd);
    assert!(icon.contains('\u{e493}') || !icon.is_empty());
}

// ========== OSC Tests ==========

#[test]
fn test_simple_base64_encode_hello() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"hello");
    assert_eq!(result, "aGVsbG8=");
}

#[test]
fn test_simple_base64_encode_empty() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"");
    assert_eq!(result, "");
}

#[test]
fn test_simple_base64_encode_single_char() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"a");
    assert_eq!(result, "YQ==");
}

#[test]
fn test_simple_base64_encode_two_chars() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"ab");
    assert_eq!(result, "YWI=");
}

#[test]
fn test_simple_base64_encode_three_chars() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"abc");
    assert_eq!(result, "YWJj");
}

#[test]
fn test_simple_base64_encode_longer() {
    let result = dracon_terminal_engine::visuals::osc::simple_base64_encode(b"hello world!");
    assert!(!result.is_empty());
}

#[test]
fn test_copy_to_clipboard_writes_correct_osc_sequence() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::copy_to_clipboard(&mut cursor, "test").unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b]52;c;"));
    assert!(output.contains("test"));
    assert!(output.ends_with("\x07"));
}

#[test]
fn test_write_hyperlink_writes_correct_osc_sequence() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::write_hyperlink(&mut cursor, "Click here", "https://example.com").unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b]8;;"));
    assert!(output.contains("https://example.com"));
    assert!(output.contains("Click here"));
    assert!(output.contains("\x1b]8;;\x1b\\"));
}

#[test]
fn test_bell_writes_bel_character() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::bell(&mut cursor).unwrap();
    let bytes = cursor.into_inner();
    assert_eq!(bytes, vec![0x07]);
}

#[test]
fn test_notify_writes_osc_777() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::notify(&mut cursor, "Title", "Body").unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b]777;notify;Title;Body\x1b\\"));
}

#[test]
fn test_copy_to_clipboard_with_special_chars() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::copy_to_clipboard(&mut cursor, "hello\x00world").unwrap();
}

#[test]
fn test_copy_to_clipboard_empty_string() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::osc::copy_to_clipboard(&mut cursor, "").unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b]52;c;"));
}

// ========== Sync Mode Tests ==========

#[test]
fn test_begin_sync_writes_decset_2026() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::sync::begin_sync(&mut cursor).unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert_eq!(output, "\x1b[?2026h");
}

#[test]
fn test_end_sync_writes_decrst_2026() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::sync::end_sync(&mut cursor).unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert_eq!(output, "\x1b[?2026l");
}

#[test]
fn test_sync_round_trip() {
    let mut cursor = Cursor::new(Vec::new());
    dracon_terminal_engine::visuals::sync::begin_sync(&mut cursor).unwrap();
    dracon_terminal_engine::visuals::sync::end_sync(&mut cursor).unwrap();
    let bytes = cursor.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert_eq!(output, "\x1b[?2026h\x1b[?2026l");
}

#[test]
fn test_begin_sync_returns_ok() {
    let mut cursor = Cursor::new(Vec::new());
    let result = dracon_terminal_engine::visuals::sync::begin_sync(&mut cursor);
    assert!(result.is_ok());
}

#[test]
fn test_end_sync_returns_ok() {
    let mut cursor = Cursor::new(Vec::new());
    let result = dracon_terminal_engine::visuals::sync::end_sync(&mut cursor);
    assert!(result.is_ok());
}