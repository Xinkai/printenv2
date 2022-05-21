use crate::args::{ColorMode, EscapeMode};
use crate::env::RecordPair;
use crate::platform_ext::u8_vec_to_string;
use crate::AppResult;
use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::io::Write;

pub struct Printer<'a> {
    pub null: bool,
    pub color: ColorMode,
    pub escape: EscapeMode,
    pub variables: Option<&'a Vec<String>>,
}

impl<'a> Default for Printer<'a> {
    fn default() -> Self {
        Self {
            null: false,
            color: ColorMode::Auto,
            escape: EscapeMode::No,
            variables: None,
        }
    }
}

enum FormatField {
    Key,
    Value,
}

impl<'a> Printer<'a> {
    fn format(&self, bytes: &[u8], field: &FormatField) -> ColoredString {
        match u8_vec_to_string(bytes) {
            Ok(string) => {
                let string = {
                    if self.escape == EscapeMode::No {
                        string
                    } else {
                        Self::escape(&string)
                    }
                };
                match field {
                    FormatField::Key => string.yellow(),
                    FormatField::Value => string.bright_white(),
                }
            }
            Err(string) => string.red(),
        }
    }

    fn escape(string: &str) -> String {
        string
            .chars()
            .map(|char| {
                if char.is_control() {
                    char.escape_default().collect()
                } else {
                    char.to_string()
                }
            })
            .collect()
    }

    fn print_with_variables(
        &self,
        record_results: &[RecordPair],
        variables: &[String],
    ) -> AppResult<Vec<u8>> {
        let mut output = Vec::new();
        let dict: HashMap<_, _> = record_results
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        for variable in variables {
            if let Some(value) = dict.get(variable.as_bytes()) {
                write!(
                    &mut output,
                    "{value}{separator}",
                    value = self.format(value, &FormatField::Value),
                    separator = if self.null { "\0" } else { "\n" },
                )?;
            }
        }
        Ok(output)
    }

    fn print_without_variables(&self, record_results: &[RecordPair]) -> AppResult<Vec<u8>> {
        let mut output = Vec::new();
        let equal_sign = "=";
        for record_result in record_results {
            write!(
                &mut output,
                "{key}{equal_sign}{value}{separator}",
                key = self.format(&record_result.0, &FormatField::Key),
                equal_sign = equal_sign.white(),
                value = self.format(&record_result.1, &FormatField::Value),
                separator = if self.null { "\0" } else { "\n" },
            )?;
        }
        Ok(output)
    }

    pub fn print(&self, record_results: &[RecordPair]) -> AppResult<Vec<u8>> {
        if self.color == ColorMode::Never {
            colored::control::set_override(false);
        }

        self.variables.map_or_else(
            || self.print_without_variables(record_results),
            |variables| self.print_with_variables(record_results, variables),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Printer;
    use crate::args::ColorMode;

    #[test]
    fn escape() {
        let cases = vec![("String", "String"), ("a\nb", "a\\nb"), ("中文", "中文")];
        for case in cases {
            assert_eq!(Printer::escape(case.0), case.1);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn null_mode() {
        use crate::{env, remote_linux_procfs};

        let printer = Printer {
            null: true,
            color: ColorMode::Never, // args.rs would set color to never if null is enabled
            ..Printer::default()
        };

        let actual = {
            let records = env::get_record_pairs_for_current_process();
            printer.print(&records).unwrap()
        };

        let expected = remote_linux_procfs::get_environment_string(std::process::id()).unwrap();
        assert_eq!(actual, expected);
    }
}
