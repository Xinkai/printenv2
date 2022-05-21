use std::fs::File;
use std::io::Read;

use crate::definition::AppResult;

#[allow(dead_code)]
pub fn get_environment_string(pid: u32) -> AppResult<Vec<u8>> {
    let mut file = File::open(format!("/proc/{}/environ", pid))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::get_environment_string;
    use crate::env::{get_record_pairs_for_current_process, parse_env_var_string};

    #[test]
    fn test_get_environment_string() {
        let env_string = get_environment_string(std::process::id()).unwrap();
        let actual = parse_env_var_string(&env_string);
        let expected = get_record_pairs_for_current_process();
        assert_eq!(actual, expected);
    }
}
