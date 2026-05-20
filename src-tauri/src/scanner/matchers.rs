use std::fs;
use std::path::Path;
use exif::Reader;

pub fn extract_exif(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = Reader::new().read_from_container(&mut bufreader)
        .map_err(|e| e.to_string())?;
    let mut exif_data = Vec::new();
    for field in exif.fields() {
        let tag = field.tag.to_string();
        let value = field.display_value().to_string();
        exif_data.push(format!("{}: {}", tag, value));
    }
    Ok(exif_data.join("; "))
}
