//! Простой файловый логгер BrVIdge.
//!
//! Перенесено из BrVIdge как наш собственный код,
//! перелицензированный в GPL-2.0-only для этого форка XCL.
//!
//! Пишет русские сообщения с уровнем и именем подсистемы.
//! Внешних зависимостей нет; файл создаётся в каталоге данных пользователя.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use brv_config::LogLevel;

/// Одна запись журнала.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Уровень.
    pub level: LogLevel,
    /// Подсистема.
    pub target: String,
    /// Сообщение.
    pub message: String,
    /// Готовая строка для вывода.
    pub line: String,
}

/// Логгер.
pub struct Logger {
    level: LogLevel,
    path: Option<PathBuf>,
    entries: Mutex<Vec<LogEntry>>,
}

impl Logger {
    /// Создать логгер. Если путь указан, создаёт каталог и файл.
    pub fn new(level: LogLevel, path: Option<PathBuf>) -> Result<Self, String> {
        if let Some(path) = &path {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("не удалось создать {parent:?}: {error}"))?;
                }
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|error| format!("не удалось открыть {path:?}: {error}"))?;
            let _ = writeln!(file, "# BrVIdge log, level={}", level.name());
        }
        Ok(Self {
            level,
            path,
            entries: Mutex::new(Vec::new()),
        })
    }

    /// Уровень логгера.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Путь к файлу журнала.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Записать сообщение. Возвращает `false`, если уровень отфильтрован.
    pub fn log(&self, level: LogLevel, target: &str, message: &str) -> Result<bool, String> {
        if level > self.level {
            return Ok(false);
        }
        let line = format!(
            "{} [{}] [{}] {}",
            timestamp(),
            level.name(),
            target,
            message
        );
        if let Some(path) = &self.path {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|error| format!("не удалось открыть {path:?}: {error}"))?;
            writeln!(file, "{line}")
                .map_err(|error| format!("не удалось записать в {path:?}: {error}"))?;
        }
        self.entries.lock().map_err(|error| error.to_string())?.push(LogEntry {
            level,
            target: target.to_owned(),
            message: message.to_owned(),
            line,
        });
        Ok(true)
    }

    /// Все записи в памяти.
    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries
            .lock()
            .map_or_else(|_| Vec::new(), |entries| entries.clone())
    }
}

/// Текущее время в формате `YYYY-MM-DD HH:MM:SS` (UTC).
pub fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    let days = seconds.div_euclid(86_400);
    let remaining = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        remaining / 3600,
        (remaining % 3600) / 60,
        remaining % 60
    )
}

/// Преобразование days since 1970-01-01 в дату (алгоритм Хинната).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + if month <= 2 { 1 } else { 0 }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_log_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("brv_log_{name}_{nanos}.log"))
    }

    #[test]
    fn filters_levels() {
        let logger = Logger::new(LogLevel::Info, None).unwrap();
        assert!(!logger.log(LogLevel::Debug, "test", "скрыто").unwrap());
        assert!(logger.log(LogLevel::Info, "test", "видно").unwrap());
        assert_eq!(logger.entries().len(), 1);
        assert_eq!(logger.entries()[0].target, "test");
    }

    #[test]
    fn writes_russian_messages_to_file() {
        let path = temp_log_path("file");
        let logger = Logger::new(LogLevel::Trace, Some(path.clone())).unwrap();
        logger.log(LogLevel::Warn, "brv_hle", "неподдерживаемый импорт").unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("неподдерживаемый импорт"));
        assert!(text.contains("[brv_hle]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn timestamp_is_readable() {
        let timestamp = timestamp();
        assert_eq!(timestamp.len(), 19);
        assert_eq!(timestamp.as_bytes()[4], b'-');
        assert_eq!(timestamp.as_bytes()[10], b' ');
    }

    #[test]
    fn creates_parent_directory() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("brv_log_dir_{nanos}"));
        let path = directory.join("nested").join("brv.log");
        Logger::new(LogLevel::Info, Some(path.clone())).unwrap();
        assert!(path.is_file());
        let _ = fs::remove_dir_all(directory);
    }
}
