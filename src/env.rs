use crate::args::KeyOrder;
use crate::platform_ext;

pub type RecordPair = (Vec<u8>, Vec<u8>);

pub fn parse_env_var_string(raw_bytes: &[u8]) -> Vec<RecordPair> {
    raw_bytes
        .split(|c| *c == 0)
        .filter_map(|record| -> Option<RecordPair> {
            record
                .iter()
                .position(|c| b'=' == *c)
                .map(|i| ((&record[..i]).to_vec(), (&record[i + 1..]).to_vec()))
        })
        .collect()
}

pub fn sort_pairs(key_order: KeyOrder, records: &mut [RecordPair]) {
    // Sort results if needed
    match key_order {
        KeyOrder::Asc => {
            records.sort_by(|a, b| a.0.cmp(&b.0));
        }
        KeyOrder::Desc => {
            records.sort_by(|a, b| b.0.cmp(&a.0));
        }
    }
}

pub fn get_record_pairs_for_current_process() -> Vec<RecordPair> {
    std::env::vars_os()
        .map(|(key, value)| {
            (
                platform_ext::os_string_to_u8_vec(&key),
                platform_ext::os_string_to_u8_vec(&value),
            )
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_records_by_env_string() {
        let cases = vec![(
            b"a=b\0c=d\0",
            vec![
                (b"a".to_vec(), b"b".to_vec()),
                (b"c".to_vec(), b"d".to_vec()),
            ],
        )];

        for case in cases {
            assert_eq!(parse_env_var_string(case.0), case.1);
        }
    }
}
