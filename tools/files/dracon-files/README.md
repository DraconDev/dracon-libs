# dracon-files

File system operations with FsCatalog for categorization and search.

## Usage

```rust
use dracon_files::{FsCatalog, FileCategory};

let catalog = FsCatalog;
let category = catalog.get_file_category(path);
let result = catalog.check_file_suitability(path, 10 * 1024 * 1024); // 10MB
```

## Key Types

- [`FsCatalog`] — main entry point
- [`FileCategory`] — Archive, Image, Script, Text, Document, Audio, Video, Other

## License

AGPL-3.0-only
