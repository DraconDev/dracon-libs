use crate::compositor::plane::{Cell, Color, Styles};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Folder,
    File,
    Rust,
    Json,
    Settings,
    Dracon,
}

pub struct IconAsset {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl Icon {
    pub fn get_asset(&self) -> IconAsset {
        match self {
            Icon::Folder => generate_folder_icon(),
            Icon::File => generate_file_icon(),
            Icon::Rust => generate_rust_icon(),
            Icon::Json => generate_json_icon(),
            Icon::Settings => generate_settings_icon(),
            Icon::Dracon => generate_dracon_icon(),
        }
    }
}

fn generate_folder_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '󰉋',
            fg: Color::Rgb(250, 200, 50),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}

fn generate_file_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '󰈔',
            fg: Color::Rgb(200, 200, 220),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}

fn generate_rust_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '',
            fg: Color::Rgb(222, 165, 132),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}

fn generate_json_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '',
            fg: Color::Rgb(100, 150, 255),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}

fn generate_settings_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '󰒓',
            fg: Color::Rgb(150, 150, 160),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}

fn generate_dracon_icon() -> IconAsset {
    let cells = vec![
        Cell {
            char: '󰊠',
            fg: Color::Rgb(255, 50, 50),
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        },
        Cell {
            char: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: true,
            skip: false,
        },
    ];
    IconAsset {
        width: 2,
        height: 1,
        cells,
    }
}