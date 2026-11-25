//! Clean-room заготовка GTA VI-специфичного слоя.
//!
//! Сейчас здесь только идентификация сборок по content hash и synthetic fixtures.
//! Ни одна реальная сборка игры не зарегистрирована: `KNOWN_BUILDS` пуст.
//! Это не заявляет о совместимости с GTA VI.

#![deny(missing_docs)]

/// Синтетический PE32+ для тестов разрешения импортов.
pub mod synthetic_pe;

/// Описание известной сборки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownBuild {
    /// Стабильный идентификатор сборки.
    pub id: &'static str,
    /// Хэш образа.
    pub hash: u64,
    /// Пояснение.
    pub notes: &'static str,
}

/// Реестр известных сборок.
///
/// Намеренно пуст: легально полученный исполняемый файл GTA VI отсутствует.
pub const KNOWN_BUILDS: &[KnownBuild] = &[];

/// Хэш FNV-1a 64 бит для идентификации образа.
pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Найти описание сборки по байтам образа.
pub fn identify(bytes: &[u8]) -> Option<&'static KnownBuild> {
    let hash = content_hash(bytes);
    KNOWN_BUILDS.iter().find(|build| build.hash == hash)
}

/// Синтетический образ для тестов (не игра).
pub fn synthetic_image(seed: u8) -> Vec<u8> {
    let mut bytes = vec![0u8; 256];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = seed.wrapping_add((index as u8).wrapping_mul(31));
    }
    bytes
}

/// Импорт синтетического заголовка.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticImport {
    /// Имя DLL.
    pub dll: String,
    /// Имя функции.
    pub function: String,
}

/// Куда направлен импорт в offline-модели.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteKind {
    /// Реализация XCL HLE.
    Hle,
    /// Передача системной DLL Windows.
    Host,
    /// Заглушка.
    Stub,
}

impl std::fmt::Display for RouteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hle => write!(f, "HLE"),
            Self::Host => write!(f, "HOST"),
            Self::Stub => write!(f, "STUB"),
        }
    }
}

fn route_kind(dll: &str, function: &str) -> RouteKind {
    match (dll, function) {
        ("kernelx.dll", "XMemAlloc")
        | ("xgameruntime.dll", "XUserGetXUID")
        | ("d3d12_xs.dll", "D3D12XboxCreateDevice") => RouteKind::Hle,
        ("kernel32.dll", _) | ("ntdll.dll", _) => RouteKind::Host,
        _ => RouteKind::Stub,
    }
}

/// Классифицировать импорты, разобранные PE-парсером XCL.
pub fn routes_from_xcl_imports(
    imports: &[xcl::pe::ImportDll],
) -> Vec<(SyntheticImport, RouteKind)> {
    let mut result = Vec::new();
    for dll in imports {
        for entry in &dll.entries {
            result.push((
                SyntheticImport {
                    dll: dll.dll.clone(),
                    function: entry.what.to_string(),
                },
                route_kind(&dll.dll, &entry.what.to_string()),
            ));
        }
    }
    result
}

/// Синтетический заголовок для проверки маршрутизации импортов.
pub struct SyntheticTitle {
    /// Имя заголовка.
    pub name: String,
    /// Импорты.
    pub imports: Vec<SyntheticImport>,
}

impl SyntheticTitle {
    /// Фикстура с типами обработчиков XCL.
    pub fn fixture() -> Self {
        Self {
            name: "brv-synthetic-gdk-title".to_owned(),
            imports: vec![
                SyntheticImport {
                    dll: "kernelx.dll".to_owned(),
                    function: "XMemAlloc".to_owned(),
                },
                SyntheticImport {
                    dll: "xgameruntime.dll".to_owned(),
                    function: "XUserGetXUID".to_owned(),
                },
                SyntheticImport {
                    dll: "d3d12_xs.dll".to_owned(),
                    function: "D3D12XboxCreateDevice".to_owned(),
                },
                SyntheticImport {
                    dll: "unknown_x.dll".to_owned(),
                    function: "NoImplementation".to_owned(),
                },
            ],
        }
    }

    /// Классифицировать импорты по offline-политике.
    pub fn routes(&self) -> Vec<(SyntheticImport, RouteKind)> {
        self.imports
            .iter()
            .map(|import| {
                (
                    import.clone(),
                    route_kind(&import.dll, &import.function),
                )
            })
            .collect()
    }

    /// Текстовый отчёт о покрытии API.
    pub fn report(&self) -> String {
        let routes = self.routes();
        let mut lines = vec![format!("Синтетический заголовок: {}", self.name)];
        for (import, kind) in &routes {
            lines.push(format!("  {}!{} → {}", import.dll, import.function, kind));
        }
        lines.push(format!(
            "  итого: HLE {}, HOST {}, STUB {}",
            routes.iter().filter(|(_, k)| *k == RouteKind::Hle).count(),
            routes.iter().filter(|(_, k)| *k == RouteKind::Host).count(),
            routes.iter().filter(|(_, k)| *k == RouteKind::Stub).count()
        ));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_builds_are_empty_by_design() {
        assert!(KNOWN_BUILDS.is_empty());
        assert!(identify(&synthetic_image(1)).is_none());
    }

    #[test]
    fn content_hash_is_deterministic() {
        let image = synthetic_image(7);
        assert_eq!(content_hash(&image), content_hash(&image));
        assert_ne!(content_hash(&image), content_hash(&synthetic_image(8)));
    }

    #[test]
    fn synthetic_title_routes_are_classified() {
        let title = SyntheticTitle::fixture();
        let routes = title.routes();
        assert_eq!(routes.len(), 4);
        assert_eq!(routes[0].1, RouteKind::Hle);
        assert_eq!(routes[3].1, RouteKind::Stub);
        let report = title.report();
        assert!(report.contains("kernelx.dll!XMemAlloc → HLE"));
        assert!(report.contains("STUB 1"));
    }

    #[test]
    fn host_route_is_available_in_policy() {
        let title = SyntheticTitle {
            name: "host-test".to_owned(),
            imports: vec![SyntheticImport {
                dll: "kernel32.dll".to_owned(),
                function: "GetTickCount".to_owned(),
            }],
        };
        assert_eq!(title.routes()[0].1, RouteKind::Host);
    }

    #[test]
    fn synthetic_image_is_not_a_real_executable() {
        let image = synthetic_image(1);
        assert!(!image.starts_with(b"MZ"));
        assert_eq!(image.len(), 256);
    }
}
