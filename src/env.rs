use crate::args::KeyOrder;
use crate::platform_ext;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::cmp::Ordering;
use std::slice::Iter;

#[derive(Eq, Debug)]
pub struct RecordPair(pub Vec<u8>, pub Vec<u8>);

impl PartialEq<Self> for RecordPair {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialOrd<Self> for RecordPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecordPair {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Env(pub Vec<RecordPair>);

impl Serialize for Env {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for RecordPair(ref k, ref v) in &self.0 {
            let key = platform_ext::u8_vec_to_string(k).unwrap_or_else(|this| this);
            let val = platform_ext::u8_vec_to_string(v).unwrap_or_else(|this| this);
            map.serialize_entry(&key, &val)?;
        }
        map.end()
    }
}

impl Env {
    pub fn iter(&self) -> Iter<'_, RecordPair> {
        self.0.iter()
    }

    pub fn filter_keys(&mut self, keys: &[String]) {
        self.0
            .retain(|item| keys.iter().any(|key| key.as_bytes() == item.0));
    }

    pub fn sort_by_key(&mut self, key_order: KeyOrder) {
        // Sort results if needed
        match key_order {
            KeyOrder::Asc => {
                self.0.sort();
            }
            KeyOrder::Desc => {
                self.0.sort_by(|a, b| b.cmp(a));
            }
        }
    }
}

#[cfg(remote_env)]
pub mod remote {
    use crate::AppResult;
    pub fn get_environment_string(pid: u32) -> AppResult<Vec<u8>> {
        #[cfg(target_os = "linux")]
        {
            crate::remote_linux_procfs::get_environment_string(pid)
        }

        #[cfg(unix_kvm)]
        {
            crate::remote_unix_kvm::get_environment_string(pid)
        }

        #[cfg(target_family = "windows")]
        {
            crate::remote_windows::get_environment_string(pid)
        }

        #[cfg(unix_apple_sysctl)]
        {
            crate::remote_apple_sysctl::get_environment_string(pid)
        }
    }

    #[test]
    fn test_get_environment_string() {
        use crate::args::ColorMode;
        use crate::printer::Printer;

        let actual = get_environment_string(std::process::id()).unwrap();
        let expected = super::Env::new();
        let printer = Printer {
            null: true,
            color: ColorMode::Never,
            ..Default::default()
        };
        assert_eq!(actual, printer.print(&expected).unwrap());
    }
}

fn parse_record_pair(record: &[u8]) -> Option<RecordPair> {
    record
        .iter()
        .position(|c| b'=' == *c)
        .map(|i| RecordPair(record[..i].to_vec(), record[i + 1..].to_vec()))
}

impl From<Vec<u8>> for Env {
    fn from(env_string: Vec<u8>) -> Self {
        Self(
            env_string
                .split(|c| *c == 0)
                .filter_map(parse_record_pair)
                .collect(),
        )
    }
}

impl Env {
    pub fn new() -> Self {
        Self(
            std::env::vars_os()
                .map(|(key, value)| {
                    RecordPair(
                        platform_ext::os_string_to_u8_vec(&key),
                        platform_ext::os_string_to_u8_vec(&value),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Env, RecordPair};
    use crate::args::{ColorMode, KeyOrder};
    use crate::Printer;

    #[test]
    fn parse_records_by_env_string() {
        let cases = vec![(
            b"a=b\0c=d\0",
            Env(vec![
                RecordPair(b"a".to_vec(), b"b".to_vec()),
                RecordPair(b"c".to_vec(), b"d".to_vec()),
            ]),
        )];

        for case in cases {
            let env_obj = Env::from(case.0.to_vec());
            assert_eq!(env_obj, case.1);
        }
    }

    #[test]
    fn sort() {
        let mut env = Env::from(Vec::from("A=111\0C=333\0B=222\0"));

        let printer = Printer {
            null: true,
            color: ColorMode::Never,
            ..Default::default()
        };

        {
            let actual = printer.print(&env).unwrap();
            assert_eq!(actual, Vec::from("A=111\0C=333\0B=222\0"));
        }

        {
            env.sort_by_key(KeyOrder::Asc);
            let actual = printer.print(&env).unwrap();
            assert_eq!(actual, Vec::from("A=111\0B=222\0C=333\0"));
        }

        {
            env.sort_by_key(KeyOrder::Desc);
            let actual = printer.print(&env).unwrap();
            assert_eq!(actual, Vec::from("C=333\0B=222\0A=111\0"));
        }
    }

    #[test]
    fn retain() {
        let mut env = Env::from(Vec::from("A=111\0C=333\0B=222\0"));
        let keys = vec!["C".to_owned()];
        env.filter_keys(&keys);
        assert_eq!(env.0.len(), 1);
        assert_eq!(env.0[0], RecordPair(Vec::from("C"), Vec::from("333")));
    }
}
