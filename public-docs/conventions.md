# Engineering Conventions

## Design Principles

- Prefer modular design and keep files focused on one responsibility.
- Apply DRY aggressively once logic repeats in a second place.
- Refactor early when design quality drops.
- Prefer predictable performance and avoid unnecessary polling.

## Rust Conventions

- Do not use `panic!` for expected failures; return typed errors.
- Minimize dependency footprint and avoid heavy runtime crates unless required.
- Use `unsafe` only where it simplifies unavoidable low-level platform interop.
- Avoid spreading explicit lifetimes unless needed by API boundaries.
- For Windows API style values, use Hungarian-style naming:
  - `sz` for zero-terminated strings
  - `s` for Rust strings
  - `n` for numbers
  - `b` for booleans
  - `p` for pointers / contiguous memory starts / function pointers
- For normal Rust domain logic, follow standard Rust API naming guidance.

## TypeScript Conventions

- Do not use `throw`.
- Avoid `try/catch` except when handling exceptions from third-party APIs.
- Use semantic `null` and `undefined` when they improve correctness.
- Do not use `!!` for boolean conversion.
- Do not use `cond && fn()` to represent control flow branches.
- Prefer `?.`, `??`, `||`, and `?.()` when suitable.
- Define functions with `function` declarations.
- For boolean fields, prefer names without `is` prefixes.
- Avoid `enum`; use `as const` + derived union types.
- Avoid anonymous inline object types in public signatures; define named interfaces.
- `any` is acceptable where strict typing adds no value.

## Formatting & Imports

- Use Prettier with:
  - 2-space indentation
  - single quotes
  - 200-character print width
  - no semicolons
  - deterministic import ordering

Suggested import order:

```text
1. ^react(-.*)?$
2. ^[a-zA-Z]
3. ^@
4. ^@[a-zA-Z]
5. ^\./(?!.*\.(css|scss)$).*
6. ^\.\./(?!.*\.(css|scss)$).*
7. \.(css|scss)$
```

## React & Frontend Structure

- Prefer component folder style:

```text
FooComponent/
  index.tsx
  index.scss
```

- Use Zustand for shared global state when needed.
- Avoid creating custom React Context-based state systems unless unavoidable.
- Avoid Tailwind as the primary styling strategy.
- Avoid inline styles except when setting `backgroundImage`.
- Prefer non-BEM class naming.

## CSS / SCSS Conventions

- Use SCSS nesting for related blocks.
- Use `cubic-bezier(0.29, 0, 0, 1)` as the default animation curve.
- Use `!important` only when necessary to override third-party styles.
- Keep comments on dedicated lines, never trailing after code.
