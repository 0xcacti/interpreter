use anyhow::Result;

pub fn load_monkey(path: String) -> Result<String> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents)
}
