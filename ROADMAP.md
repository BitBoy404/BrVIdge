# Дорожная карта миграции BrVIdge → XCL fork

Этот файл описывает **миграционные фазы M0–M8** основного репозитория
`BrVIdge-XCL`. Он не заменяет upstream-документ `docs/ROADMAP.md` и не
изменяет приостановленную Phase 7 в архиве `G:\Projects\Opencode\BrVIdge`.

Статус на 25 ноября 2026 года: выбрана стратегия **B пользователя**
(форк существующего проекта), что соответствует варианту **C** в
`PROJECT_AUDIT.md`.

## M0 — Fork и юридическая база

**Цель:** зафиксировать upstream, лицензию и provenance.

- [x] Создан локальный fork `xcl-dev/xcl` на коммите
      `e9e94ce665d16bb0e2d2e4cb6b37e149f0d27721`.
- [x] История upstream сохранена; `origin` переименован в `upstream`.
- [x] `LICENSE` (GPL-2.0-only) и `THIRD_PARTY.md` сохранены без изменений.
- [x] Юридическая матрица зафиксирована в `MIGRATION.md` и `PROJECT_AUDIT.md`.
- [x] Игровые файлы, ключи и утечки не использовались.

**Ворота M0:** пройдены. Подтверждение: `git log -1`, `git remote -v`,
`cargo test --workspace` → 922 passed, 0 failed, 42 ignored.

## M1 — Hardware baseline

**Цель:** подтвердить requirements на Ryzen 5 5600 / RTX 5060 8 ГБ /
32 ГБ RAM / Windows 10 Pro 22H2.

- [x] Установлен D3D12 Agility SDK `Microsoft.Direct3D.D3D12 1.620.2`
      в `%LOCALAPPDATA%\XCL\D3D12\` (DLL не добавлены в репозиторий).
- [x] DXC из Windows SDK: 1.9.2602.17.
- [x] Wave32-адаптация изучена по `pso.rs`, `shaders.rs`, `recomp/mod.rs`;
      отдельный патч `crates/xcl/src` не потребовался.
- [x] Бюджет UMA и число потоков зафиксированы в `crates/brv_target`.
- [ ] Upstream синхронизация: upstream уже ушёл на
      `568e7db7729c8fa6c8f0f4b32d798faaf11462bb`; слияние не выполнялось.

**Ворота M1:** локальная часть пройдена; upstream sync остаётся открытой
задачей и не блокирует синтетическую разработку.

## M2 — Сохранение BrVIdge как архива

**Цель:** не потерять исследования и тесты без переноса несовместимого кода.

- [x] BrVIdge не удалялся; код не переписывался.
- [x] Документация и отчёты скопированы в `docs/brvidge-archive/`.
- [x] `PROJECT_AUDIT.md`, `docs/reports/xcl-comparison.md`, `phase6.md`,
      `phase7.md` сохранены.
- [x] В BrVIdge изменены только статусные записи в `README.md`/`ROADMAP.md`
      о приостановке Phase 7.

**Ворота M2:** пройдены. Исходники BrVIdge остаются в исходном виде.

## M3 — Перенос собственных модулей

**Цель:** перенести только наш код с проверенной лицензией.

- [x] `crates/brv_config`, `crates/brv_log`, `crates/brv_diag`
      перелицензированы в GPL-2.0-only и подключены к workspace.
- [x] Let-chains заменены на вложенные `if` из-за edition 2021 XCL.
- [x] Тесты: 16 passed, 0 failed.
- [x] GPL-3-код VIctory/fork, Apache-2 scarlett-layer не переносился.
- [x] `crates/xcl/src` не изменялся.

**Ворота M3:** пройдены.

## M4 — Синтетические фикстуры GTA VI и отчётность

**Цель:** развивать GTA VI-направление без игры и без утечек.

- [x] `crates/gta6`: `KNOWN_BUILDS` пуст, `content_hash`, `identify`,
      `SyntheticTitle`, `synthetic_pe` (PE32+ с тремя консольными импортами).
- [x] `crates/brv_target`: профиль RTX 5060, бюджет UMA, feature checklist.
- [x] `tools/brv_target_report`: отчёт без запуска игры.
- [x] Все проверки на синтетических данных.
- [ ] Реальные идентификаторы сборок — заблокированы отсутствием легальной копии.

**Ворота M4:** синтетическая часть пройдена; реальные сборки не проверяются.

## M5 — Синхронизация с upstream

**Цель:** безопасно подтянуть изменения `xcl-dev/xcl`.

- [x] `git fetch upstream`: refs обновлены, HEAD `568e7db`.
- [x] Сравнение `e9e94ce..upstream/main`: **1019 коммитов**, 102 файла,
      +24 615 / −1 068 строк. Большинство изменений — `crates/xcl/src`,
      README и тесты.
- [x] Проверено: `LICENSE` и `THIRD_PARTY.md` не изменились; в `Cargo.toml`
      сохранились `edition = 2021`, `license = "GPL-2.0-only"` и исходный
      `members` upstream.
- [x] Новые upstream-файлы, которые **не** переносятся автоматически:
      `hle/playstation.rs`, `source_hash.rs`, `d3d12_resource_and_sync.rs`,
      coverage-gate scripts.
- [ ] Merge **не выполняется**: по `AGENTS.md` правки `crates/xcl/src` вне
      тестов требуют `scripts/smoke.ps1 -Game <folder>` на двух сборках;
      пользователь запретил запуск игры и у нас нет легального тайтла.
      Дополнительно в рабочем дереве есть наши uncommitted изменения.
- [ ] Не запускать игру и не использовать `XCL_SAMPLE_*` без легальных файлов.

**Ворота M5:** оценка выполнена, merge заблокирован. Форк остаётся на
`e9e94ce`. Это осознанное ограничение, а не заявление о синхронизации.

## M6 — Clean-room GTA VI-слой

**Цель:** расширить `crates/gta6` без кода VIctory.

- [x] `gta6::synthetic_pe` строит PE32+ с parametrized imports;
      парсер `xcl::pe::Pe` читает их без запуска игры.
- [x] `gta6::routes_from_xcl_imports` классифицирует разобранные импорты
      по offline-политике (HLE/HOST/STUB).
- [x] Сценарии `boot`, `series-s`, `host` (CLI `--scenario`) с отдельными
      `report.json`/`xcl.log`; некорректные списки импортов → `Err`.
- [ ] Хуки и идентификация сборок на основе собственных наблюдений.
- [ ] Юридическая проверка BCPack-permission внутри XCL.
- [ ] Никаких игровых файлов.

**Ворота M6:** частично — синтетический PE и маршрутизация проверены;
реальные сборки не проверяются, `KNOWN_BUILDS` пуст.

## M7 — Интеграция и диагностика

**Цель:** связать новые модули с XCL runtime без запуска игры.

- [x] Интеграция `gta6` с `xcl::pe`: synthetic PE → imports → routes
      (unit-тест в `gta6/src/synthetic_pe.rs`).
- [x] `brv_target_report --report <path>` пишет `report.json` через реальный
      тип `xcl::report::Report`: `imports_by_source` HLE 3 / STUB 1,
      `stubbed_imports` содержит `unknown_x.dll!NoImplementation`.
- [x] `brv_target_report --log <path>` пишет строки через реальный
      `xcl::log::init` / `xlog_info!`: synthetic title, профиль и каждый импорт.
- [x] `report.json` заполняет `environment` через `xcl::preflight::Check`,
      `exit_code = 0`, `crash = None` и note про нулевые ошибки и бюджет UMA.
- [x] Unit-тест `environment_checks_describe_target`.
- [x] `FrameBudget`, `FrameTimeline` и сценарии `menus|world|stall` с
      сводкой (`--timeline`), пишутся в `xcl.log` и `report.json`.
- [x] `write_synthetic_report` принимает bytes и возвращает `Err` на битом
      PE без записи файла; тест `invalid_pe_returns_error_without_writing_report`.
- [ ] Профилирование на доступном железе только синтетическими тестами.

**Ворота M7:** пройдены в синтетическом объёме — PE → imports → routes →
`xcl.log` + `report.json` (типы XCL), environment checks и контроль бюджета.
`XCL_SAMPLE_*` не использовались; игра не запускалась.

## M8 — Легальный title-тест

**Цель:** только если пользователь предоставит законно полученную копию.

- [ ] Не начинать без легальных файлов.
- [ ] Не обходить DRM и не расшифровывать XVC/MSIXVC.
- [ ] Запуск в игре — только по явному разрешению пользователя.

**Ворота M8:** заблокированы.

## Правила обновления

- Фаза помечается `done` только после фактического выполнения и тестов.
- Команды не создаются без просьбы.
- `crates/xcl/src` меняется только после проверки `AGENTS.md` и при
  возможности прогнать `scripts/smoke.ps1`; без игры это невозможно.
- Отчёты о тестах указывают точную команду и счётчики.

