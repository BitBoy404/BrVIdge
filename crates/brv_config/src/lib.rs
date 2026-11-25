//! Конфигурация BrVIdge и профили Xbox Series X|S.
//!
//! Перенесено из исследовательского проекта BrVIdge как наш собственный
//! код и перелицензировано в GPL-2.0-only для этого форка XCL.
//!
//! Парсер поддерживает небольшое подмножество TOML: секции, пары
//! `ключ = значение`, строки в кавычках, целые числа и комментарии `#`.
//! Это сделано намеренно, чтобы не добавлять внешнюю зависимость
//! для нескольких десятков полей.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;
use std::fs;
use std::path::Path;

/// Ошибка конфигурации.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Некорректный синтаксис.
    Syntax {
        /// Номер строки.
        line: usize,
        /// Текст ошибки.
        detail: String,
    },
    /// Неизвестный ключ или секция.
    UnknownKey(String),
    /// Неподдерживаемое значение.
    InvalidValue {
        /// Ключ.
        key: String,
        /// Значение.
        value: String,
    },
    /// Ошибка файловой системы.
    Io(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { line, detail } => {
                write!(f, "синтаксис конфигурации, строка {line}: {detail}")
            }
            Self::UnknownKey(key) => write!(f, "неизвестный ключ конфигурации: {key}"),
            Self::InvalidValue { key, value } => {
                write!(f, "недопустимое значение {key}: {value}")
            }
            Self::Io(text) => write!(f, "ошибка файла конфигурации: {text}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Профиль консоли.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleProfile {
    /// Идентификатор профиля.
    pub id: String,
    /// Оптимальная для GPU память (MiB).
    pub gpu_optimal_mib: u32,
    /// Стандартная память (MiB).
    pub standard_mib: u32,
    /// Игровые ядра CPU (публичные спецификации).
    pub game_cpu_cores: u32,
}

impl ConsoleProfile {
    /// Профиль Series X.
    pub fn series_x() -> Self {
        Self {
            id: "series-x".to_owned(),
            gpu_optimal_mib: 10 * 1024,
            standard_mib: 3 * 1024 + 512,
            game_cpu_cores: 7,
        }
    }

    /// Профиль Series S.
    pub fn series_s() -> Self {
        Self {
            id: "series-s".to_owned(),
            gpu_optimal_mib: 0,
            standard_mib: 7776,
            game_cpu_cores: 4,
        }
    }

    /// Найти профиль по имени.
    pub fn by_id(id: &str) -> Option<Self> {
        match id {
            "series-x" => Some(Self::series_x()),
            "series-s" => Some(Self::series_s()),
            _ => None,
        }
    }
}

/// Уровень логирования.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Только ошибки.
    Error,
    /// Предупреждения и ошибки.
    Warn,
    /// Информационные сообщения.
    Info,
    /// Отладочные сообщения.
    Debug,
    /// Трассировка вызовов.
    Trace,
}

impl LogLevel {
    /// Разобрать имя уровня.
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warn" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }

    /// Имя уровня.
    pub fn name(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

/// Конфигурация BrVIdge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Выбранный профиль консоли.
    pub console: String,
    /// Каталог данных пользователя.
    pub data_dir: String,
    /// Размер UMA в MiB (ограничен 8 ГБ VRAM хоста).
    pub uma_mib: u32,
    /// Максимум фоновых потоков трансляции.
    pub worker_threads: u32,
    /// Уровень логирования.
    pub log_level: LogLevel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            console: "series-x".to_owned(),
            data_dir: "data".to_owned(),
            uma_mib: 4096,
            worker_threads: 2,
            log_level: LogLevel::Info,
        }
    }
}

impl Config {
    /// Профиль, соответствующий конфигурации.
    pub fn profile(&self) -> Option<ConsoleProfile> {
        ConsoleProfile::by_id(&self.console)
    }

    /// Проверить согласованность значений.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.profile().is_none() {
            return Err(ConfigError::InvalidValue {
                key: "console".to_owned(),
                value: self.console.clone(),
            });
        }
        if self.uma_mib == 0 || self.uma_mib > 6144 {
            return Err(ConfigError::InvalidValue {
                key: "uma_mib".to_owned(),
                value: self.uma_mib.to_string(),
            });
        }
        if self.worker_threads == 0 || self.worker_threads > 8 {
            return Err(ConfigError::InvalidValue {
                key: "worker_threads".to_owned(),
                value: self.worker_threads.to_string(),
            });
        }
        if self.data_dir.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "data_dir".to_owned(),
                value: self.data_dir.clone(),
            });
        }
        Ok(())
    }

    /// Сериализовать в поддерживаемое подмножество TOML.
    pub fn to_toml(&self) -> String {
        format!(
            "# Конфигурация BrVIdge\n[general]\nconsole = \"{}\"\ndata_dir = \"{}\"\n\n[limits]\numa_mib = {}\nworker_threads = {}\n\n[log]\nlevel = \"{}\"\n",
            self.console,
            self.data_dir,
            self.uma_mib,
            self.worker_threads,
            self.log_level.name().to_ascii_lowercase()
        )
    }

    /// Загрузить конфигурацию из файла.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path)
            .map_err(|error| ConfigError::Io(format!("{path:?}: {error}")))?;
        Self::parse(&text)
    }

    /// Сохранить конфигурацию в файл.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| ConfigError::Io(format!("{parent:?}: {error}")))?;
            }
        }
        fs::write(path, self.to_toml())
            .map_err(|error| ConfigError::Io(format!("{path:?}: {error}")))
    }

    /// Разобрать конфигурацию из строки.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let mut section = String::new();
        for (index, raw_line) in text.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_owned();
                if !matches!(section.as_str(), "general" | "limits" | "log") {
                    return Err(ConfigError::Syntax {
                        line: line_number,
                        detail: format!("неизвестная секция [{section}]"),
                    });
                }
                continue;
            }
            let (key, value) = line.split_once('=').ok_or(ConfigError::Syntax {
                line: line_number,
                detail: "ожидается '='".to_owned(),
            })?;
            let key = key.trim();
            let value = value.trim();
            let full_key = format!("{section}.{key}");
            match full_key.as_str() {
                "general.console" => config.console = parse_string(value, line_number)?,
                "general.data_dir" => config.data_dir = parse_string(value, line_number)?,
                "limits.uma_mib" => config.uma_mib = parse_number(value, line_number)?,
                "limits.worker_threads" => {
                    config.worker_threads = parse_number(value, line_number)?;
                }
                "log.level" => {
                    let name = parse_string(value, line_number)?;
                    config.log_level = LogLevel::parse(&name).ok_or(ConfigError::InvalidValue {
                        key: full_key,
                        value: name,
                    })?;
                }
                _ => return Err(ConfigError::UnknownKey(full_key)),
            }
        }
        config.validate()?;
        Ok(config)
    }
}

fn parse_string(value: &str, line: usize) -> Result<String, ConfigError> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(ConfigError::Syntax {
            line,
            detail: "ожидается строка в кавычках".to_owned(),
        });
    }
    Ok(value[1..value.len() - 1].to_owned())
}

fn parse_number(value: &str, line: usize) -> Result<u32, ConfigError> {
    value.parse::<u32>().map_err(|_| ConfigError::Syntax {
        line,
        detail: format!("ожидается целое число, получено '{value}'"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let config = Config::default();
        config.validate().unwrap();
        assert_eq!(config.profile().unwrap().id, "series-x");
        assert_eq!(config.uma_mib, 4096);
    }

    #[test]
    fn profiles_match_public_specs() {
        assert_eq!(ConsoleProfile::series_x().gpu_optimal_mib, 10 * 1024);
        assert_eq!(ConsoleProfile::series_x().game_cpu_cores, 7);
        assert_eq!(ConsoleProfile::series_s().gpu_optimal_mib, 0);
        assert_eq!(ConsoleProfile::series_s().standard_mib, 7776);
        assert!(ConsoleProfile::by_id("unknown").is_none());
    }

    #[test]
    fn parses_and_roundtrips_toml_subset() {
        let source = r#"
# тестовая конфигурация
[general]
console = "series-s"
data_dir = "data_test"
[limits]
uma_mib = 2048
worker_threads = 3
[log]
level = "debug"
"#;
        let config = Config::parse(source).unwrap();
        assert_eq!(config.console, "series-s");
        assert_eq!(config.uma_mib, 2048);
        assert_eq!(config.log_level, LogLevel::Debug);
        let reparsed = Config::parse(&config.to_toml()).unwrap();
        assert_eq!(reparsed, config);
    }

    #[test]
    fn invalid_config_is_rejected() {
        assert!(Config::parse("[general]\nconsole = \"series-pro\"").is_err());
        assert!(Config::parse("[limits]\numa_mib = 999999").is_err());
        assert!(Config::parse("[log]\nlevel = \"verbose\"").is_err());
        assert!(Config::parse("[general]\nunknown = 1").is_err());
        assert!(Config::parse("[bad]\nfoo = 1").is_err());
        assert!(Config::parse("[general]\nconsole = series-x").is_err());
    }

    #[test]
    fn log_levels_are_ordered() {
        assert!(LogLevel::Error < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Trace);
        assert_eq!(LogLevel::parse("TRACE"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::Trace.name(), "TRACE");
    }

    #[test]
    fn save_and_load_file() {
        let directory = std::env::temp_dir().join(format!("brv_config_test_{}", std::process::id()));
        let path = directory.join("brv.toml");
        let config = Config {
            console: "series-s".to_owned(),
            data_dir: "data".to_owned(),
            uma_mib: 3072,
            worker_threads: 2,
            log_level: LogLevel::Warn,
        };
        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded, config);
        let _ = fs::remove_dir_all(directory);
    }
}
