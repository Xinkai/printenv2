use std::fs::File;
use std::io::Read;

use crate::definition::AppResult;

pub fn get_environment_string(pid: u32) -> AppResult<Vec<u8>> {
    let mut file = File::open(format!("/proc/{}/environ", pid))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
