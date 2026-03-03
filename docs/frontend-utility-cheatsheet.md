# Frontend Utility Cheatsheet (Project-Specific)

This is the set of utility classes currently available in `assets/public/grade-o-matic.min.css`.
Source of truth for templates.

## Layout

- `flex`
- `flex-wrap`
- `items-center`
- `justify-between`
- `mx-auto`
- `max-w-4xl`
- `min-h-screen`
- `min-w-full`
- `overflow-hidden`

## Spacing

- `gap-2`, `gap-3`
- `mt-1`, `mt-3`, `mt-4`
- `mb-3`
- `p-4`, `p-5`
- `px-2`, `px-3`, `px-4`
- `py-1`, `py-2`, `py-8`

## Typography

- `text-xs`, `text-sm`, `text-lg`, `text-xl`, `text-3xl`
- `font-medium`, `font-semibold`, `font-bold`
- `tracking-tight`
- `text-left`, `text-center`
- `text-white`
- `text-slate-500`, `text-slate-600`, `text-slate-700`, `text-slate-900`

## Colors, Borders, Effects

- `bg-white`, `bg-slate-50`, `bg-slate-100`, `bg-slate-900`
- `border`, `border-slate-200`, `border-slate-300`
- `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-full`
- `shadow-sm`
- `hover:bg-slate-100`, `hover:bg-slate-700`

## Existing component classes

- `auth-input` (plus `auth-input:focus`)
- `status-badge`
- Nav-related classes:
    - `site-nav-wrap`
    - `site-nav-inner`
    - `site-brand`
    - `site-nav-menu`
    - `site-nav-link`
    - `site-nav-toggle`
    - `site-nav-dropdown`
    - `site-nav-panel`
    - `site-nav-panel-item`

## Common patterns

- Card wrapper:
    - `rounded-xl border border-slate-200 bg-white p-5 shadow-sm`
- Button (primary):
    - `rounded-md bg-slate-900 px-3 py-2 text-sm font-medium text-white hover:bg-slate-700`
- Button (secondary):
    - `rounded-md border border-slate-300 px-3 py-2 text-sm font-medium hover:bg-slate-100`

## Important gotcha

If a class from Tailwind docs does nothing, it likely is not compiled in this project stylesheet.
Example: `justify-end` is currently not present, while `justify-between` is present.

When in doubt, check `assets/public/grade-o-matic.min.css` first.
