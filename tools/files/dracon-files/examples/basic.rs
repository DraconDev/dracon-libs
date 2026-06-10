use std::path::Path;

use dracon_files::{FileInspectContract, FileSuitabilityContract, FsCatalog};

fn main() -> anyhow::Result<()> {
    let catalog = FsCatalog;

    let test_file = Path::new("/tmp/test.txt");
    std::fs::write(test_file, "hello world")?;

    let category = catalog.get_file_category(test_file);
    println!("Category for {}: {:?}", test_file.display(), category);

    let result = catalog.check_file_suitability(test_file, 10 * 1024 * 1024);
    println!("Suitability (10MB limit): {:?}", result);

    std::fs::remove_file(test_file)?;

    Ok(())
}
