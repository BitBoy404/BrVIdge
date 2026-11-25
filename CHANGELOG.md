# Журнал изменений

Формат: обратный хронологический порядок. Все записи подтверждены
реально выполненными действиями и тестами.

## [Unreleased] — 25 ноября 2026

### Решение и основа

- Выбрана стратегия **B** из текущей задачи (в `PROJECT_AUDIT.md` — вариант C):
  основной репозиторий — форк `xcl-dev/xcl`, BrVIdge остаётся архивом.
- Зафиксирован локальный fork на коммите `e9e94ce` с сохранёнными
  `LICENSE` (GPL-2.0-only), `THIRD_PARTY.md` и историей upstream.
- Обнаружено расхождение с upstream: удалённый HEAD сейчас
  `568e7db7729c8fa6c8f0f4b32d798faaf11462bb`. Слияние не выполнялось.

### Инфраструктура и среда

- Добавлен `MIGRATION.md` с лицензионной матрицей и планом перехода.
- Установлен D3D12 Agility SDK `Microsoft.Direct3D.D3D12 1.620.2` в
  `%LOCALAPPDATA%\XCL\D3D12\`; DLL не включены в репозиторий.
- Упавший на Windows 10 GPU-тест
  `gpu_raw_window_view_reaches_its_top` проходит после установки Agility SDK.
- Добавлен `ROADMAP.md` с фазами миграции M0–M8; Phase 7 BrVIdge не закрывалась.

### Перенесённые модули

- `crates/brv_config`, `crates/brv_log`, `crates/brv_diag`: наш собственный
  код перелицензирован в GPL-2.0-only, подключён к workspace.
- В `Cargo.toml` workspace добавлены записи для этих и новых crates.
- Let-chains заменены на вложенные `if` для edition 2021 XCL.

### Новые синтетические компоненты

- `crates/gta6`: `content_hash`, `identify`, `KNOWN_BUILDS` (пуст),
  `SyntheticTitle`, `synthetic_pe` (PE32+ с тремя консольными импортами).
- `crates/brv_target`: профиль RTX 5060, бюджет UMA, feature checklist.
- `tools/brv_target_report`: отчёт о профиле и синтетических данных
  без запуска игры.

### Архив

- Документация BrVIdge скопирована в `docs/brvidge-archive/` (13 файлов),
  включая `PROJECT_AUDIT.md` и отчёты по фазам.

### Тестирование

```
cargo build --workspace                        → успешно
cargo test --workspace -- --test-threads=4     → 922 passed, 0 failed, 42 ignored
cargo run -p brv_target_report                 → отчёт без запуска игры
```

DXC: `1.9.2602.17` из Windows SDK. Тесты на реальных играх не запускались;
`XCL_SAMPLE_*` не задавались.

### M6/M7: синтетическая интеграция

- `gta6/src/synthetic_pe.rs` теперь создаёт корректный OriginalFirstThunk
  с import-by-name; XCL-парсер `xcl::pe::Pe::parse(..., Layout::File)`
  читает три консольных DLL без запуска игры.
- Добавлен `gta6::routes_from_xcl_imports`: разобранные импорты
  классифицируются по offline-политике HLE/HOST/STUB.
- Тесты `gta6`: 10 passed, 0 failed. Полный workspace:
  **924 passed, 0 failed, 42 ignored**.
- `cargo clippy -p gta6` не проходит из-за 472 upstream-ошибок в `xcl`
  (tree не rustfmt/clippy-clean по `AGENTS.md`). Эти ошибки не наши и не
  исправлялись.

### Очистка Claude Code

- Удалён `CLAUDE.md`: он содержал только ссылки на `AGENTS.md` и
  `CONTRIBUTING.md` и дублировал их правила; уникальной информации нет.
- `AGENTS.md`: формулировка прочитана как «AI coding agents» без перечисления
  конкретных инструментов; пример в разделе AI use обобщён.
- `CONTRIBUTING.md`: убрана ссылка на удалённый `CLAUDE.md` и конкретный
  пример с Claude Code; требование discloses tool/model сохранено.
- Сохранены: `AGENTS.md`, `README.md`, `ROADMAP.md`, `SECURITY.md`,
  `THIRD_PARTY.md`, `HANDOFF.md`, исходники и тесты.
- Сохранён факт в архиве аудита про внешний fork (историческое observation)
  и тестовые данные `scripts/test_pr_check.py` — это не конфигурация Claude.

### Pre-alpha readiness

- Добавлен `PRE_ALPHA.md`: checklist готовности, ограничения и список того,
  что нужно от пользователя для создания GitHub-репозитория и коммита.
- В `README.md` добавлена преамбула о форке и pre-alpha статусе; текст
  upstream README сохранён ниже без изменений.
- Проверено: `cargo build --release --workspace` успешно; репозиторий не
  содержит игровых файлов, ключей или DLL Agility SDK.
- Коммиты и remote `origin` не создавались.

### M7: синтетические frame-timing сценарии

- `brv_target`: `FrameBudget`, `FrameTimeline` и сценарии `menus`, `world`,
  `stall`; сводка содержит min/max/avg и число кадров вне бюджета.
- CLI `brv_target_report --timeline` пишет сводку в `xcl.log` и `report.json`.
- `cargo test --workspace` → 930 passed, 0 failed, 42 ignored.
- Игра не запускалась.

### M6: расширение синтетических сценариев

- `gta6::synthetic_pe::build_with_imports` принимает до 8 пар `(DLL, функция)`
  и возвращает `Err` на пустой/слишком длинный список.
- Добавлены сценарии `boot_imports`, `series_s_imports`, `host_imports`;
  CLI `brv_target_report --scenario` пишет отдельные `report.json`/`xcl.log`.
- `series-s`: HLE 3 / STUB 2; `host`: HLE 1 / HOST 2 / STUB 1.
- Тесты `scenarios_are_parsed_by_xcl_and_have_expected_routes` и
  `invalid_import_lists_are_refused`.
- `cargo test --workspace` → 928 passed, 0 failed, 42 ignored.

### M7: отчётность XCL на синтетических данных

- `tools/brv_target_report` получил флаг `--report <path>` и пишет JSON
  через реальный тип `xcl::report::Report` (сериализация `serde_json`).
- Синтетический PE32+ дополнен четвёртым импортом
  `unknown_x.dll!NoImplementation`; исправлены RVA, чтобы не перекрывать
  dll names, thunks и function names.
- Проверено:
  ```
  cargo run -p brv_target_report -- --report H:\Temp\opencode\brv-synthetic-report.json
  → imports_by_source: HLE 3, STUB 1; stubbed_imports: NoImplementation
  ```
- Добавлен `--log <path>`: строки пишутся через `xcl::log::init` и
  `xlog_info!` (синтетический заголовок, профиль, каждый импорт).
- `report.json` теперь заполняет `environment` через
  `xcl::preflight::Check`, `exit_code = 0`, `crash = None` и note о нулевых
  ошибках и бюджете UMA.
- Unit-тест `environment_checks_describe_target`.
- `write_synthetic_report` рефакторен под `&[u8]`; битый PE возвращает
  `Err` и не создаёт файл (тест `invalid_pe_returns_error_without_writing_report`).
- `cargo test --workspace` → 926 passed, 0 failed, 42 ignored.
- Игра не запускалась; `XCL_SAMPLE_*` не использовались.

### M5: оценка upstream-синхронизации

- `git fetch upstream` подтянул `568e7db`; расхождение с форком:
  1019 коммитов, 102 файла, +24 615 / −1 068 строк.
- `LICENSE`, `THIRD_PARTY.md` и базовые поля `Cargo.toml` upstream не изменились.
- Merge не выполнялся: по `AGENTS.md` правки `crates/xcl/src` требуют
  `scripts/smoke.ps1 -Game <folder>`, а запуск игры запрещён и легального
  тайтла нет; в рабочем дереве также есть локальные uncommitted-изменения.
- Форк остаётся на `e9e94ce`; фаза M5 помечена как «оценка выполнена,
  merge заблокирован».

### Ограничения

- `crates/xcl/src` не изменялся.
- Коммиты и PR не создавались.
- `origin` для fork-remote не настраивался по указанию пользователя.
- Игровые файлы, ключи и DRM-обход не использовались.

