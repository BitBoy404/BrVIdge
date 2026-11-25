# Pre-alpha readiness

Документ фиксирует, что именно подготовлено к публикации форка на GitHub
и что ещё не готово. Статус: **pre-alpha**, игра не запускалась.

## 1. Что проверено

Команды и результаты на машине: Ryzen 5 5600, RTX 5060 8 ГБ, 32 ГБ RAM,
Windows 10 Pro 22H2, Rust 1.99.1, DXC 1.9.2602.17.

```
cargo build --workspace                       → успешно
cargo build --release --workspace             → успешно
cargo test --workspace -- --test-threads=4    → 930 passed, 0 failed, 42 ignored
cargo run -p brv_target_report -- --scenario series-s --timeline stall \
  --report <path> --log <path>                → synthetic report + xcl.log
```

Игра не запускалась; `XCL_SAMPLE_*` не использовались; тяжёлые GPU/игровые
тесты не проводились.

## 2. Состав форка

- upstream `xcl-dev/xcl` на `e9e94ce`, лицензия GPL-2.0-only;
  `LICENSE`, `THIRD_PARTY.md`, история сохранены.
- Наши собственные crates, перелицензированные в GPL-2.0-only:
  `brv_config`, `brv_log`, `brv_diag`, `brv_target`, `gta6`.
- Утилита `tools/brv_target_report` с режимами
  `--scenario boot|series-s|host` и `--timeline menus|world|stall`.
- Синтетический PE32+-генератор, offline route-отчёт, environment checks
  и frame-timing проходят через реальные типы XCL (`pe`, `report`, `preflight`, `log`).

## 3. Готовность к публикации

| Требование | Статус | Подтверждение |
|---|---|---|
| Лицензия и provenance | готово | `LICENSE` GPL-2, `THIRD_PARTY.md`, заголовки в наших crates |
| Отсутствие игровых файлов/ключей | готово | проверка каталога: запрещённых расширений и файлов >5 МБ вне `.git`/`target` нет |
| Agility SDK не в репозитории | готово | DLL лежат в `%LOCALAPPDATA%\XCL\D3D12\` |
| `Cargo.lock` зафиксирован | готово | обновлён сборкой workspace |
| CI | готово | `.github/workflows/ci.yml` собирает и тестирует workspace на Windows без GPU/игры |
| README описывает форк | готово | преамбула в `README.md` + этот документ |
| Дорожная карта | готово | `ROADMAP.md` (M0–M8), `MIGRATION.md`, `CHANGELOG.md` |
| Стабильная сборка | готово | debug + release |
| Тесты | готово | 930 passed, 0 failed |

## 4. Что НЕ готово (pre-alpha ограничения)

- **M5**: upstream ушёл на `568e7db`, merge не выполнялся; требуется
  `scripts/smoke.ps1 -Game <folder>` для правок `crates/xcl/src`, что
  невозможно без игры и запрещено пользователем.
- **M6**: `KNOWN_BUILDS` пуст, реальные build identities не известны.
- **M8**: легальный title-тест заблокирован — нет легальной копии игры.
- GTA VI gameplay не достигнут ни в одном проекте.
- `cargo clippy -p gta6` не проходит из-за 472 upstream-ошибок в `xcl`
  (дерево upstream не clippy-clean). Наши crates не исправляли upstream.
- `crates/xcl/src` не изменялся; все новые модули живут вне upstream-кода.

## 5. Что нужно от пользователя для публикации

1. Репозиторий на GitHub (или разрешение его создать) и его URL.
2. Имя и видимость (public/private).
3. Авторизация: GitHub CLI (`gh auth login`) или токен с правом `repo`;
   токен не стоит передавать в чат — лучше выполнить вход локально.
4. Явное разрешение на `git commit` и `git push` в этот репозиторий.
5. Подтверждение, что ветка `main` и remote `origin` именно новые;
   текущий `upstream` уже указывает на `xcl-dev/xcl`.
6. Подтверждение юридической схемы: наши вклады перелицензированы
   в GPL-2.0-only, GPL-3-код VIctory не переносился.

## 6. Локальный коммит (когда будет разрешение)

```
git -C G:\Projects\Opencode\BrVIdge-XCL status --short
git -C G:\Projects\Opencode\BrVIdge-XCL add -A
git -C G:\Projects\Opencode\BrVIdge-XCL commit -m "Fork: add synthetic compatibility harness and pre-alpha docs"
git -C G:\Projects\Opencode\BrVIdge-XCL remote add origin <URL>
git -C G:\Projects\Opencode\BrVIdge-XCL push -u origin main
```

Пока этого не делал: коммитов и remote `origin` нет.

