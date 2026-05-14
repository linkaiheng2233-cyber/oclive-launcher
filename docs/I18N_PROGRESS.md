# I18N progress (oclive-launcher)

## Locale wiring

- **Entry**: `src/main.ts` — `app.use(i18n)`.
- **Aligned with oclivenewnew**: `src/i18n/index.ts` shares `LOCALE_PREF_KEY = "oclive.appLocale"`, persisted preference (`system` | `zh-CN` | `en-US`), `resolveLocaleTag`, `fallbackLocale: "zh-CN"`.

## Phase 0 — CJK inventory

Command: `rg -l "\\p{Han}" --glob "*.vue" --glob "*.ts" src`

Notable files: `App.vue`, `components/HelpHint.vue`, `lib/launcherHints.ts`, `lib/rolePackCreatorMessage.ts`, announcements copy, locale tables.

## Build

- `npm run build`
