use crate::args::{ColorMode, EscapeMode};
use crate::env::{Env, RecordPair};
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

    fn print_with_variables(&self, env: &Env, variables: &[String]) -> AppResult<Vec<u8>> {
        let mut output = Vec::new();
        let mut dict: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        for RecordPair(key, value) in env.iter() {
            assert_eq!(dict.insert(key.clone(), value.clone()), None);
        }

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

    fn print_without_variables(&self, env: &Env) -> AppResult<Vec<u8>> {
        let mut output = Vec::new();
        let equal_sign = "=";
        for record_result in env.iter() {
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

    pub fn print(&self, env: &Env) -> AppResult<Vec<u8>> {
        if self.color == ColorMode::Never {
            colored::control::set_override(false);
        }
        self.variables.map_or_else(
            || self.print_without_variables(env),
            |variables| self.print_with_variables(env, variables),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Printer;
    use crate::args::{ColorMode, EscapeMode};
    use crate::env::Env;

    #[test]
    fn escape() {
        let cases = vec![("String", "String"), ("a\nb", "a\\nb"), ("中文", "中文")];
        for case in cases {
            assert_eq!(Printer::escape(case.0), case.1);
        }
    }

    #[test]
    fn print_escape() {
        let env = Env::from(Vec::from("LINEBREAK=\r\n\0TAB=\t\0"));

        let printer = Printer {
            color: ColorMode::Never,
            escape: EscapeMode::Yes,
            ..Default::default()
        };

        let actual = printer.print(&env).unwrap();
        assert_eq!(actual, Vec::from("LINEBREAK=\\r\\n\nTAB=\\t\n"));
    }

    #[test]
    fn with_variables() {
        let env = Env::from(Vec::from("VAR1=foo\0VAR2=bar\0"));
        let variables = vec!["VAR1".into()];
        let printer = Printer {
            color: ColorMode::Never,
            variables: Some(&variables),
            ..Default::default()
        };

        let actual = printer.print(&env).unwrap();
        assert_eq!(actual, Vec::from("foo\n"));
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn invalid_utf8() {
        use colored::Colorize;

        let env = Env::from(vec![
            0x56, 0x41, 0x4c, 0x3d, 0x54, 0x65, 0x73, 0x74, 0xc3, 0x28, 0x00,
        ]);

        let printer = Printer {
            color: ColorMode::Always,
            ..Default::default()
        };
        let actual = printer.print(&env).unwrap();
        assert_eq!(
            actual,
            Vec::from(format!(
                "{}{}{}{}",
                "VAL".yellow(),
                "=",
                "Test\\xc3(".red(),
                "\n"
            ))
        );
    }
}
