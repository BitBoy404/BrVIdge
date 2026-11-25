//! Профиль целевой машины и бюджет ресурсов.
//!
//! Значения выбраны для конфигурации: Ryzen 5 5600, RTX 5060 8 ГБ,
//! 32 ГБ RAM, Windows 10/11. Это рекомендации, не измеренные FPS.

#![deny(missing_docs)]

/// Профиль целевой машины.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetProfile {
    /// Видеопамять в MiB.
    pub vram_mib: u32,
    /// Оперативная память в MiB.
    pub ram_mib: u32,
    /// Логические потоки CPU.
    pub cpu_threads: u32,
    /// GPU использует 32-линейные волны.
    pub wave32: bool,
}

impl TargetProfile {
    /// Профиль основного тестового ПК.
    pub fn rtx5060() -> Self {
        Self {
            vram_mib: 8 * 1024,
            ram_mib: 32 * 1024,
            cpu_threads: 12,
            wave32: true,
        }
    }

    /// Рекомендованный бюджет UMA.
    ///
    /// Ограничиваем половиной VRAM и 4 ГБ, чтобы оставить место под
    /// render targets и драйвер на 8-ГБ карте.
    pub fn uma_budget_mib(&self) -> u32 {
        (self.vram_mib / 2).min(4096)
    }

    /// Число фоновых потоков трансляции.
    pub fn worker_threads(&self) -> u32 {
        let spare = self.cpu_threads.saturating_sub(4);
        spare.clamp(1, 2)
    }

    /// Требуется ли эмуляция wave64 на 32-линейном GPU.
    pub fn needs_wave64_emulation(&self) -> bool {
        self.wave32
    }

    /// Достаточно ли RAM для типичного заголовка GDK (~15 ГБ commit в меню).
    pub fn has_headroom_for_title(&self) -> bool {
        self.ram_mib >= 24 * 1024
    }
}

/// Требование к среде выполнения.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureRequirement {
    /// Название требования.
    pub name: &'static str,
    /// Выполнено ли требование.
    pub satisfied: bool,
    /// Пояснение.
    pub detail: &'static str,
}

/// Проверить требования для конфигурации пользователя.
///
/// `agility_sdk` и `windows_11` передаёт вызывающая сторона: этот crate
/// не обращается к файловой системе и не запускает GPU-код.
pub fn feature_checklist(
    profile: &TargetProfile,
    agility_sdk: bool,
    windows_11: bool,
) -> Vec<FeatureRequirement> {
    vec![
        FeatureRequirement {
            name: "AVX2",
            satisfied: true,
            detail: "Ryzen 5 5600 поддерживает AVX2",
        },
        FeatureRequirement {
            name: "DirectX 12",
            satisfied: true,
            detail: "RTX 5060 поддерживает D3D12",
        },
        FeatureRequirement {
            name: "Shader Model 6.6",
            satisfied: agility_sdk || windows_11,
            detail: "на Windows 10 нужен D3D12 Agility SDK",
        },
        FeatureRequirement {
            name: "Mesh shaders",
            satisfied: true,
            detail: "RTX 5060 (Blackwell) поддерживает mesh shaders",
        },
        FeatureRequirement {
            name: "wave64",
            satisfied: !profile.wave32,
            detail: "на NVIDIA wave64 эмулируется двумя половинами",
        },
        FeatureRequirement {
            name: "VRAM budget",
            satisfied: profile.vram_mib >= 8192,
            detail: "8 ГБ минимум; больше → запас для render targets",
        },
        FeatureRequirement {
            name: "RAM headroom",
            satisfied: profile.has_headroom_for_title(),
            detail: "32 ГБ достаточно с запасом",
        },
    ]
}

/// Бюджет кадра в миллисекундах.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameBudget {
    /// Целевой FPS.
    pub fps: u32,
    /// Максимальное время кадра, ms.
    pub max_ms: u32,
}

impl FrameBudget {
    /// Создать бюджет из целевого FPS.
    pub fn from_fps(fps: u32) -> Result<Self, String> {
        if fps == 0 {
            return Err("fps must be > 0".to_owned());
        }
        Ok(Self {
            fps,
            max_ms: 1000 / fps,
        })
    }
}

/// Сводка синтетической последовательности кадров.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSummary {
    /// Число кадров.
    pub count: usize,
    /// Минимальный интервал, ms.
    pub min_ms: u32,
    /// Максимальный интервал, ms.
    pub max_ms: u32,
    /// Средний интервал, ms (целочисленное деление).
    pub average_ms: u32,
    /// Кадры, превысившие бюджет.
    pub over_budget: usize,
}

/// Синтетическая временная шкала кадров.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameTimeline {
    /// Интервалы между кадрами, ms.
    pub intervals_ms: Vec<u32>,
}

impl FrameTimeline {
    /// Создать шкалу из интервалов.
    pub fn new(intervals_ms: Vec<u32>) -> Self {
        Self { intervals_ms }
    }

    /// Шкала меню ~30 fps.
    pub fn menus() -> Self {
        Self::new(vec![33, 34, 33, 33, 32])
    }

    /// Шкала мира ~5 fps.
    pub fn world() -> Self {
        Self::new(vec![200, 210, 190, 200, 205])
    }

    /// Шкала с одним зависшим кадром.
    pub fn stall() -> Self {
        Self::new(vec![16, 120, 16, 16, 17])
    }

    /// Посчитать сводку относительно бюджета.
    pub fn summary(&self, budget: &FrameBudget) -> FrameSummary {
        if self.intervals_ms.is_empty() {
            return FrameSummary {
                count: 0,
                min_ms: 0,
                max_ms: 0,
                average_ms: 0,
                over_budget: 0,
            };
        }
        let min_ms = *self.intervals_ms.iter().min().unwrap();
        let max_ms = *self.intervals_ms.iter().max().unwrap();
        let sum: u32 = self.intervals_ms.iter().sum();
        FrameSummary {
            count: self.intervals_ms.len(),
            min_ms,
            max_ms,
            average_ms: sum / self.intervals_ms.len() as u32,
            over_budget: self
                .intervals_ms
                .iter()
                .filter(|value| **value > budget.max_ms)
                .count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtx5060_profile_matches_machine() {
        let profile = TargetProfile::rtx5060();
        assert_eq!(profile.vram_mib, 8192);
        assert_eq!(profile.ram_mib, 32768);
        assert_eq!(profile.cpu_threads, 12);
        assert!(profile.wave32);
    }

    #[test]
    fn budgets_are_conservative_for_8gb_vram() {
        let profile = TargetProfile::rtx5060();
        assert_eq!(profile.uma_budget_mib(), 4096);
        assert_eq!(profile.worker_threads(), 2);
        assert!(profile.needs_wave64_emulation());
        assert!(profile.has_headroom_for_title());
    }

    #[test]
    fn smaller_vram_reduces_uma() {
        let profile = TargetProfile {
            vram_mib: 4096,
            ..TargetProfile::rtx5060()
        };
        assert_eq!(profile.uma_budget_mib(), 2048);
    }

    #[test]
    fn feature_checklist_reflects_windows_10_without_agility() {
        let profile = TargetProfile::rtx5060();
        let list = feature_checklist(&profile, false, false);
        let sm66 = list.iter().find(|f| f.name == "Shader Model 6.6").unwrap();
        assert!(!sm66.satisfied);
        let wave64 = list.iter().find(|f| f.name == "wave64").unwrap();
        assert!(!wave64.satisfied);
        assert_eq!(list.len(), 7);
    }

    #[test]
    fn feature_checklist_passes_with_agility() {
        let profile = TargetProfile::rtx5060();
        let list = feature_checklist(&profile, true, false);
        assert!(list.iter().all(|f| f.name == "wave64" || f.satisfied));
    }

    #[test]
    fn frame_budget_rejects_zero_fps() {
        assert!(FrameBudget::from_fps(0).is_err());
        assert_eq!(FrameBudget::from_fps(30).unwrap().max_ms, 33);
        assert_eq!(FrameBudget::from_fps(5).unwrap().max_ms, 200);
    }

    #[test]
    fn frame_timeline_summaries_are_deterministic() {
        let budget30 = FrameBudget::from_fps(30).unwrap();
        let menus = FrameTimeline::menus().summary(&budget30);
        assert_eq!(menus.count, 5);
        assert_eq!(menus.average_ms, 33);
        assert_eq!(menus.over_budget, 1); // 34 ms > 33 ms

        let stall = FrameTimeline::stall().summary(&budget30);
        assert_eq!(stall.max_ms, 120);
        assert_eq!(stall.over_budget, 1);

        let empty = FrameTimeline::new(vec![]).summary(&budget30);
        assert_eq!(empty.count, 0);
        assert_eq!(empty.average_ms, 0);
    }

    #[test]
    fn low_thread_count_still_has_one_worker() {
        let profile = TargetProfile {
            cpu_threads: 4,
            ..TargetProfile::rtx5060()
        };
        assert_eq!(profile.worker_threads(), 1);
    }
}
