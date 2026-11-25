# Миграция BrVIdge → XCL fork

Документ описывает текущее состояние основного репозитория после решения,
принятого по результатам `PROJECT_AUDIT.md` (25 ноября 2026 года).

Выбрана стратегия **B текущей задачи**: переход на существующий проект.
В `PROJECT_AUDIT.md` тот же выбор назван **вариантом C** (так как он
рассматривал четыре стратегии A–D). Не путать с вариантом B аудита — форком VIctory.

Актуальность upstream проверена повторно: `git ls-remote` показывает HEAD
`568e7db7729c8fa6c8f0f4b32d798faaf11462bb`, тогда как локальный форк
зафиксирован на `e9e94ce`. Upstream ушёл вперёд; слияние не выполнялось и
будет рассматриваться отдельной фазой M5 в `ROADMAP.md`.

Оценка M5 выполнена: расхождение 1019 коммитов, 102 файла,
+24 615 / −1 068 строк; `LICENSE`, `THIRD_PARTY.md` и базовые поля
`Cargo.toml` не изменились. Merge заблокирован требованием `AGENTS.md` о smoke-тесте на реальном
тайтле и запретом запускать игру. Форк остаётся на `e9e94ce`.

## 1. Текущий upstream и копия

- Upstream: `https://github.com/xcl-dev/xcl`
- Зафиксированный коммит: `e9e94ce665d16bb0e2d2e4cb6b37e149f0d27721`
  (2026-11-24 22:00 +0100, `Merge PR #1321`).
- Локальный форк: `G:\Projects\Opencode\BrVIdge-XCL`.
- Удалённый `origin` из клона переименован в `upstream`; `origin` для нового
  fork-remote будет настроен после создания удалённого репозитория.
- Лицензия форка: **GPL-2.0-only** (`LICENSE`, `Cargo.toml`).
- `THIRD_PARTY.md` сохранён без изменений: обязательно сохранять provenance
  AMD ISA/Mesa/LLVM/AnyPS5/FFmpeg и персональное разрешение на BCPack от VIctory.

## 2. Роль BrVIdge после миграции

`G:\Projects\Opencode\BrVIdge` остаётся:

- исследовательской веткой;
- архивом архитектурных решений (`RESEARCH.md`, `ARCHITECTURE.md`,
  `FEASIBILITY.md`, `RISKS.md`, `docs/reports/`);
- набором синтетических тестов и идей, которые можно переносить как наш
  собственный код.

Код и конфигурация BrVIdge в этом шаге **не удалялись и не переписывались**.

## 3. Baseline P0 (проверено в этом репозитории)

Команды:

```powershell
cargo build --workspace
cargo test --workspace -- --test-threads=4
```

Результаты:

- `cargo build --workspace` — успешно (2 предупреждения Rust 2024 о deprecated
  `fetch_update` в upstream-коде; не являются ошибкой).
- `cargo test --workspace` — **842 passed, 1 failed, 22 ignored**.
- Упавший тест: `gpu::recomp::heap_window_tests::gpu_raw_window_view_reaches_its_top`,
  причина `HRESULT(0x80070057)` в `crates/xcl/src/gpu/recomp/heap_window_tests.rs:100`.
- Причина среды: на Windows 10 Pro 22H2 in-box `D3D12Core.dll` поддерживает
  только SM 6.5; несколько GPU-тестов честно печатают
  `SKIPPED ...: NVIDIA GeForce RTX 5060: no SM 6.6 (on Windows 10, set
  XCL_D3D12_DIR to a D3D12 Agility SDK folder)`.
- Agility SDK (`D3D12Core.dll`) в системе не найден. Это **не регрессия форка**:
  код не изменялся, ошибка вызвана отсутствием компонента среды.

Действие: не маскировать тест как ignored; установить Agility SDK или
перейти на Windows 11, затем повторить baseline.

## 4. Ограничения лицензий (обязательные до любого переноса кода)

| Источник | Лицензия | Можно ли перенести в этот GPL-2 форк |
|---|---|---|
| XCL upstream | GPL-2.0-only | да, основа форка |
| Наш собственный код BrVIdge (`brv_config`, `brv_log`, `brv_diag`, тесты) | изначально GPL-3, но наш код | да, после явной перелицензии в GPL-2.0-only |
| VIctory / fxceless fork | GPL-3.0-or-later | **нет** — только идеи и факты, не код |
| scarlett-layer | Apache-2.0 | **нет** без отдельной проверки (Apache-2/GPL-2 conflict) |
| Xbox2Windows | MIT | да, с сохранением уведомления |
| AMD ISA / Mesa / silicon-rans | MIT | да |
| LLVM tests | Apache-2 WITH LLVM-exception | да по исключению |
| FFmpeg | LGPL-2.1+ | да |
| BCPack-код VIctory внутри XCL | GPL-3 + персональное разрешение | требуется юридическая проверка |

Юридических гарантий нет. До объединения кода из разных лицензий нужна
отдельная проверка.

## 5. Что переносим из BrVIdge

- `crates/brv_config`, `crates/brv_log`, `crates/brv_diag` — перенесены как
  наши собственные модули и перелицензированы в GPL-2.0-only; подключены к
  workspace. Из-за edition 2021 в XCL let-chains заменены на вложенные `if`.
- Синтетические тесты PE/RVA/релокаций и asset-policy тесты остаются в
  BrVIdge как архив; в форк перенесены только три административных модуля.
- Документация BrVIdge скопирована в `docs/brvidge-archive/` как исторический
  архив (13 файлов, включая `PROJECT_AUDIT.md`).

### Статус P1 (проверено)

```
cargo build --workspace                       → успешно
cargo test -p brv_config -p brv_log -p brv_diag → 16 тестов, 0 failed
```

### Agility SDK и чистый baseline P0

Установлен официальный NuGet `Microsoft.Direct3D.D3D12 1.620.2`:

- `D3D12Core.dll` и `d3d12SDKLayers.dll` скопированы в
  `%LOCALAPPDATA%\XCL\D3D12\` (путь, который XCL читает по README);
- бинарники **не** добавлены в репозиторий;
- лицензия пакета — Microsoft Software License Terms; `distributable files.txt`
  явно разрешает распространение этих двух DLL;
- DXC из Windows SDK: 1.9.2602.17.

После установки:

```
cargo test --workspace -- --test-threads=4
→ итого 930 passed, 0 failed, 42 ignored
```

Рост: 908 → 915 (`brv_target`, `gta6`) → 919 (тесты и `brv_target_report`)
→ 922 (синтетический PE32+) → 924 (парсер XCL + offline routes)
→ 925 (environment checks и `--log`) → 926 (обработка битых PE)
→ 928 (parametrized сценарии boot/series-s/host) → 930 (frame-timing).

Ранее упавший `gpu_raw_window_view_reaches_its_top` теперь проходит.
Это подтверждает, что причина была в среде (SM 6.5 in-box D3D12), а не
в коде форка.

## 6. Что НЕ переносим

- `brv_gpu::console_list` (собственный формат записей, не D3D12.X).
- `brv_formats::tiling` (упрощённая модель, не GFX10).
- `brv_shader` (11 инструкций) — вместо него используется `gpu/recomp` XCL.
- GPL-3-код VIctory и форка — только идеи/факты.
- Apache-2.0-код scarlett-layer — только идеи/тестовые подходы.

## 6.1 Добавленные заготовки P2

- `crates/gta6` — clean-room слой идентификации сборок: `KNOWN_BUILDS` пуст,
  `content_hash`, `identify`, `synthetic_image`. 3 теста. Никакой реальной
  сборки GTA VI нет; совместимость не заявляется.
- `crates/brv_target` — профиль RTX 5060/32 ГБ/12 потоков с консервативными
  бюджетами: `uma_budget_mib() = 4096`, `worker_threads() = 2`,
  `needs_wave64_emulation() = true`. 4 теста. Это рекомендации, не замеры FPS.

Оба crates добавлены в workspace; `cargo build --workspace` проходит.

`tools/brv_target_report` — консольная утилита, которая печатает профиль
RTX 5060, бюджет UMA, requirements checklist и отчёт по синтетическому
GTA6-заголовку. Проверка:

```
cargo run -p brv_target_report
→ UMA 4096 MiB, 2 worker, wave64 emulation true,
  SM 6.6 OK (Agility SDK), wave64 НЕТ, реальная сборка не найдена
```

Утилита **не загружает и не запускает игру**.

`gta6/src/synthetic_pe.rs` строит минимальный PE32+ AMD64 с тремя
консольными импортами (`kernelx`, `xgameruntime`, `d3d12_xs`) полностью из
собственных констант; 3 теста проверяют заголовки и строки. Это fixture для
будущих offline-тестов разрешения импортов, не игровой файл.

## 6.2 Wave32-адаптация по исходникам XCL (без правок `crates/xcl/src`)

Изучено, но не изменялось:

- `crates/xcl/src/gpu/recomp/pso.rs:754 split_wave64_forced()` — env
  `XCL_SPLIT_WAVE64=1` нужен только чтобы проверить wave32-путь на GPU,
  который умеет 64 lanes.
- `pso.rs:768 wave_range()` / `:781 wave32_only_for_device()` — на устройстве
  определяется через `D3D12_FEATURE_D3D12_OPTIONS1`; для RTX 5060 split-form
  включается автоматически, ручной флаг не нужен.
- `pso.rs:763 graphics_wave32()` и `recomp/mod.rs:1443` — для wave64 VS/PS
  используется `upper_lanes`, так как D3D12 не позволяет задать WaveSize для
  графики.
- `recomp/mod.rs:1184,1209` — `wave_hazards` и `geometry_wave32_hazards`
  описывают случаи, которые остаются неточными на 32-lane GPU.
- `shaders.rs:829,944` — split уже учитывается при выборе формы перевода.

Вывод для целевой машины: адаптация встроена в XCL; отдельный патч в upstream
не требуется. Наши собственные модули (`brv_target`) лишь фиксируют ожидаемое
поведение в профиле и тестах. По правилам `AGENTS.md` правки `crates/xcl/src`
потребовали бы `scripts/smoke.ps1 -Game <folder>`, что невозможно без игры и
запрещено пользователем. Поэтому `crates/xcl/src` не тронут.

## 7. Следующие шаги

1. Установить D3D12 Agility SDK или использовать Windows 11, повторить baseline.
2. Завершить перенос модулей BrVIdge и прогнать тесты.
3. Настроить `origin` будущего fork-remote.
4. Создать GTA VI-слой clean-room и synthetic fixtures.
5. Адаптировать под RTX 5060 8 ГБ / wave32 / 32 ГБ RAM.
