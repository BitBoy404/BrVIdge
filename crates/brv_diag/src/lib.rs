//! Диагностика BrVIdge: отчёт о среде и проверка активов репозитория.
//!
//! Перенесено из BrVIdge как наш собственный код,
//! перелицензированный в GPL-2.0-only для этого форка XCL.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fs;
use std::path::{Path, PathBuf};

use brv_config::Config;

/// Сводка по GPU, передаётся из `brv_gpu`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSummary {
    /// Имя адаптера.
    pub description: String,
    /// Видеопамять в MiB.
    pub dedicated_video_mib: u64,
}

/// Отчёт о среде.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemReport {
    /// Операционная система.
    pub os: String,
    /// Архитектура.
    pub arch: String,
    /// Доступные потоки CPU.
    pub cpu_threads: usize,
    /// GPU.
    pub gpu: Option<GpuSummary>,
    /// Путь к DXC.
    pub dxc: Option<PathBuf>,
    /// Профиль консоли.
    pub console: String,
    /// Бюджет UMA.
    pub uma_mib: u32,
    /// Фоновые потоки.
    pub worker_threads: u32,
    /// Уровень логирования.
    pub log_level: String,
}

impl SystemReport {
    /// Собрать отчёт.
    pub fn collect(
        config: &Config,
        gpu: Option<GpuSummary>,
        dxc: Option<PathBuf>,
    ) -> Self {
        Self {
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
            cpu_threads: std::thread::available_parallelism().map_or(1, |value| value.get()),
            gpu,
            dxc,
            console: config.console.clone(),
            uma_mib: config.uma_mib,
            worker_threads: config.worker_threads,
            log_level: config.log_level.name().to_owned(),
        }
    }

    /// Текстовый отчёт.
    pub fn to_text(&self) -> String {
        let mut text = String::from("Отчёт о среде BrVIdge\n");
        let _ = std::fmt::Write::write_fmt(
            &mut text,
            format_args!(
                "  ОС:                     {}\n  Архитектура:            {}\n  Потоки CPU:             {}\n",
                self.os, self.arch, self.cpu_threads
            ),
        );
        match &self.gpu {
            Some(gpu) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut text,
                    format_args!(
                        "  GPU:                    {} ({} MiB VRAM)\n",
                        gpu.description, gpu.dedicated_video_mib
                    ),
                );
            }
            None => text.push_str("  GPU:                    не обнаружен\n"),
        }
        match &self.dxc {
            Some(path) => {
                let _ = std::fmt::Write::write_fmt(
                    &mut text,
                    format_args!("  DXC:                    {}\n", path.display()),
                );
            }
            None => text.push_str("  DXC:                    не найден\n"),
        }
        let _ = std::fmt::Write::write_fmt(
            &mut text,
            format_args!(
                "  Профиль консоли:        {}\n  Бюджет UMA:             {} MiB\n  Фоновые потоки:         {}\n  Уровень логирования:    {}\n",
                self.console, self.uma_mib, self.worker_threads, self.log_level
            ),
        );
        text
    }
}

/// Экранировать строку для JSON.
pub fn json_escape(text: &str) -> String {
    let mut output = String::new();
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => {
                let _ = std::fmt::Write::write_fmt(&mut output, format_args!("\\u{:04X}", value as u32));
            }
            value => output.push(value),
        }
    }
    output
}

/// Построить JSON-отчёт из сводки и произвольных строковых полей.
pub fn report_json(report: &SystemReport, fields: &[(String, String)]) -> String {
    let mut json = String::from("{\n");
    let values = [
        ("os", report.os.clone()),
        ("arch", report.arch.clone()),
        ("cpu_threads", report.cpu_threads.to_string()),
        (
            "gpu",
            report
                .gpu
                .as_ref()
                .map_or_else(|| "не обнаружен".to_owned(), |gpu| gpu.description.clone()),
        ),
        (
            "vram_mib",
            report
                .gpu
                .as_ref()
                .map_or_else(|| "0".to_owned(), |gpu| gpu.dedicated_video_mib.to_string()),
        ),
        (
            "dxc",
            report
                .dxc
                .as_ref()
                .map_or_else(|| "не найден".to_owned(), |path| path.display().to_string()),
        ),
        ("console", report.console.clone()),
        ("uma_mib", report.uma_mib.to_string()),
        ("worker_threads", report.worker_threads.to_string()),
        ("log_level", report.log_level.clone()),
    ];
    for (key, value) in values {
        let _ = std::fmt::Write::write_fmt(
            &mut json,
            format_args!(
                "  \"{}\": \"{}\",\n",
                json_escape(key),
                json_escape(&value)
            ),
        );
    }
    for (index, (key, value)) in fields.iter().enumerate() {
        let comma = if index + 1 == fields.len() { "" } else { "," };
        let _ = std::fmt::Write::write_fmt(
            &mut json,
            format_args!(
                "  \"{}\": \"{}\"{}\n",
                json_escape(key),
                json_escape(value),
                comma
            ),
        );
    }
    json.push_str("}\n");
    json
}

/// Нарушение политики активов.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Путь к файлу.
    pub path: PathBuf,
    /// Причина.
    pub reason: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.reason)
    }
}

/// Расширения файлов, которые не должны попадать в репозиторий.
pub const FORBIDDEN_EXTENSIONS: &[&str] = &[
    "xvc", "msixvc", "rpf", "bk2", "xvd", "pak", "utoc", "ucas", "xbe", "wad",
];

/// Имена файлов, разрешённые политикой несмотря на размер.
pub const ALLOWED_LARGE_FILES: &[&str] = &["bcpack-tables.bin"];

/// Максимальный размер обычного файла в репозитории.
pub const MAX_FILE_BYTES: u64 = 512 * 1024;

/// Проверить каталог на соответствие политике активов.
pub fn check_dir(root: &Path) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();
    check_dir_recursive(root, root, &mut violations)?;
    Ok(violations)
}

fn check_dir_recursive(
    root: &Path,
    directory: &Path,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("не удалось прочитать {directory:?}: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("ошибка записи в {directory:?}: {error}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if matches!(name.as_str(), "target" | "data" | ".git") {
                continue;
            }
            check_dir_recursive(root, &path, violations)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let extension = path
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if FORBIDDEN_EXTENSIONS.contains(&extension.as_str()) {
            violations.push(Violation {
                path: path.clone(),
                reason: format!("запрещённое расширение .{extension}"),
            });
        }
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("не удалось получить размер {path:?}: {error}"))?;
        if metadata.len() > MAX_FILE_BYTES && !ALLOWED_LARGE_FILES.contains(&name.as_str()) {
            violations.push(Violation {
                path: path.clone(),
                reason: format!("файл больше {} байт", MAX_FILE_BYTES),
            });
        }
        let mut header = [0u8; 4];
        use std::io::Read;
        let mut file = fs::File::open(&path)
            .map_err(|error| format!("не удалось открыть {path:?}: {error}"))?;
        let read = file
            .read(&mut header)
            .map_err(|error| format!("не удалось прочитать {path:?}: {error}"))?;
        if read == 4 {
            let magic = &header[..4];
            if magic == b"MZ\x90\x00" || magic.starts_with(b"MZ") {
                violations.push(Violation {
                    path: path.clone(),
                    reason: "исполняемый файл PE (MZ)".to_owned(),
                });
            } else if magic == b"DXBC" || magic == b"DDS " || magic == b"RIFF" {
                violations.push(Violation {
                    path: path.clone(),
                    reason: format!("бинарный ресурс ({})", String::from_utf8_lossy(magic)),
                });
            }
        }
        let _ = root;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("brv_diag_{name}_{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn report_contains_config_values() {
        let config = Config::default();
        let report = SystemReport::collect(
            &config,
            Some(GpuSummary {
                description: "Test GPU".to_owned(),
                dedicated_video_mib: 8192,
            }),
            None,
        );
        let text = report.to_text();
        assert!(text.contains("Test GPU"));
        assert!(text.contains("8192 MiB"));
        assert!(text.contains("series-x"));
        assert!(text.contains("4096 MiB"));
        assert!(text.contains("DXC:                    не найден"));
    }

    #[test]
    fn report_json_is_well_formed() {
        let report = SystemReport::collect(&Config::default(), None, None);
        let json = report_json(
            &report,
            &[("exit_code".to_owned(), "0".to_owned())],
        );
        assert!(json.starts_with("{\n"));
        assert!(json.trim_end().ends_with('}'));
        assert!(json.contains("\"exit_code\": \"0\""));
        assert_eq!(json_escape("a\"b\\c"), "a\\\"b\\\\c");
    }

    #[test]
    fn clean_repository_has_no_violations() {
        let directory = temp_dir("clean");
        fs::write(directory.join("README.md"), "# тест\n").unwrap();
        assert!(check_dir(&directory).unwrap().is_empty());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn detects_forbidden_extension_and_large_file() {
        let directory = temp_dir("bad");
        fs::write(directory.join("game.xvc"), "x").unwrap();
        fs::write(directory.join("big.bin"), vec![0u8; (MAX_FILE_BYTES + 1) as usize]).unwrap();
        let violations = check_dir(&directory).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|value| value.reason.contains("xvc")));
        assert!(violations.iter().any(|value| value.reason.contains("больше")));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn detects_pe_and_dxbc_magic() {
        let directory = temp_dir("magic");
        fs::write(directory.join("game.exe"), b"MZ\x90\x00rest").unwrap();
        fs::write(directory.join("shader.dxbc"), b"DXBCrest").unwrap();
        let violations = check_dir(&directory).unwrap();
        assert_eq!(violations.len(), 2);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn checks_nested_directories_and_skips_target() {
        let directory = temp_dir("nested");
        fs::create_dir_all(directory.join("crates/test")).unwrap();
        fs::create_dir_all(directory.join("target/debug")).unwrap();
        fs::write(directory.join("crates/test/a.xvc"), "x").unwrap();
        fs::write(directory.join("target/debug/b.xvc"), "x").unwrap();
        let violations = check_dir(&directory).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].path.ends_with("crates/test/a.xvc"));
        let _ = fs::remove_dir_all(directory);
    }
}
