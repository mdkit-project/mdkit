# Contributing to mdkit

Thanks for considering a contribution. mdkit is small and intentionally
stays small; the bar for what gets merged is "does this fit the dispatch
model and improve real-world extraction quality." Below is what you
need to know.

## What we want

- **Format support.** New `Extractor` implementations for formats not
  yet covered. See the README roadmap for which backends are landing
  in which version.
- **Quality fixes.** A failing extraction on a real-world document
  (with a redacted-but-representative test fixture if possible).
- **Performance.** Profile-driven speedups, especially in the dispatch
  path or the in-process backends.
- **Platform-native polish.** Better Vision.framework / Windows.Media.Ocr
  integration; alternative pure-Rust OCR backends.

## What we're skeptical of

- New top-level public API surfaces. The `Extractor` trait + `Engine`
  are the contract; extending them needs strong justification.
- Format support that duplicates Pandoc when Pandoc handles it well
  (DOCX/PPTX/EPUB/RTF/ODT/LaTeX). If you can show measurable quality
  or speed gains, great — but "I want a pure-Rust DOCX path" by
  itself isn't enough.
- Network calls. Extractors run on-device by default; anything that
  reaches a remote service must be opt-in via a feature flag and
  obvious in the API.

## Pull-request checklist

Before opening a PR, please verify:

- [ ] `cargo build --all-features` succeeds.
- [ ] `cargo test --all-features` passes.
- [ ] `cargo clippy --all-features -- -D warnings` is clean.
- [ ] `cargo fmt --check` shows no diff.
- [ ] Public API additions have rustdoc comments.
- [ ] If you're adding a backend, the README's feature-flag table is
      updated.
- [ ] If you're adding a backend that pulls in a sidecar binary or
      large model, the size cost is documented in the README.

## Test fixtures

Real-world documents make the best fixtures. If you can include a
DOCX/PDF/whatever that triggers your bug, please:

- Strip identifying information before committing.
- Keep the file under 100 KB if possible. For larger fixtures, store
  them out-of-tree and reference them in `/tests/fixtures/large/`
  (gitignored).

## Coding style

- Idiomatic Rust per `rustfmt` defaults.
- No `unsafe` in core dispatch code. Backend FFI may use `unsafe` but
  must wrap it in safe abstractions and document invariants.
- Public items get doc comments; backend-internal helpers don't need
  them but appreciate them.
- Errors are typed via [`error::Error`](src/error.rs); avoid `String`
  errors at API boundaries.

## Licensing

By contributing you agree your contributions will be licensed under
the same dual MIT/Apache-2.0 terms as the rest of the project.
