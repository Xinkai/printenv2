use crate::args::KeyOrder;

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
