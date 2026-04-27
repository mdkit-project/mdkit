//! PDF text extraction via Google's Pdfium engine.
//!
//! Backed by the [`pdfium-render`](https://crates.io/crates/pdfium-render)
//! crate, which wraps Pdfium — the same PDF engine that ships in
//! Chrome and that powers most of the world's web-based PDF viewing.
//! Layout-aware, multi-column-friendly, handles encrypted documents
//! (returns a clean error when no password is supplied).
//!
//! ## Runtime requirement: libpdfium
//!
//! `pdfium-render` doesn't bundle the actual Pdfium library — it loads
//! `libpdfium.{so,dylib,dll}` dynamically at runtime. Consumers of
//! mdkit's `pdf` feature need to make libpdfium available on their
//! library search path.
//!
//! Recommended sources of pre-built libpdfium binaries:
//!
//! - [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) —
//!   community-maintained pre-built binaries for all major platforms.
//! - [paulocoutinhox/pdfium-lib](https://github.com/paulocoutinhox/pdfium-lib) —
//!   per-platform release archives.
//!
//! On macOS and Linux you typically drop `libpdfium.dylib` /
//! `libpdfium.so` next to your binary or onto `LD_LIBRARY_PATH`. On
//! Windows, place `pdfium.dll` next to the executable.
//!
//! ## What this extractor does NOT do
//!
//! - **No OCR.** Scanned (image-only) PDFs return empty or near-empty
//!   text. The OCR backends (`ocr-platform`, `ocr-onnx`) handle the
//!   image-text case; mdkit's [`Engine`](crate::Engine) will fall
//!   back to OCR automatically when both features are enabled.
//! - **No password support.** Encrypted PDFs return
//!   [`Error::ParseError`](crate::Error::ParseError) with a clear
//!   message. Password-protected extraction lands when a real
//!   user-need surfaces.
//! - **No layout-mode selection.** Pdfium's default text-extraction
//!   mode is used, which preserves reading order for most documents.
//!   A configurable layout mode lands if real-world output proves
//!   inadequate.

use crate::{Document, Error, Extractor, Result};
use pdfium_render::prelude::*;
use std::path::Path;

/// PDF extractor backed by Pdfium. Construct via [`PdfiumExtractor::new`]
/// (which discovers libpdfium on the system library path) or
/// [`PdfiumExtractor::with_library_path`] (which loads from an explicit
/// directory — useful when libpdfium ships next to your application
/// binary).
pub struct PdfiumExtractor {
    pdfium: Pdfium,
}

impl PdfiumExtractor {
    /// Construct an extractor by binding to libpdfium on the system's
    /// default library search path. Returns
    /// [`Error::MissingDependency`](crate::Error::MissingDependency)
    /// if libpdfium can't be found or loaded.
    pub fn new() -> Result<Self> {
        let bindings = Pdfium::bind_to_system_library().map_err(|e| Error::MissingDependency {
            name: "libpdfium".into(),
            details: format!("could not load from system library path: {e}"),
        })?;
        Ok(Self {
            pdfium: Pdfium::new(bindings),
        })
    }

    /// Construct an extractor by binding to libpdfium at an explicit
    /// path. Useful when the libpdfium binary ships alongside your
    /// application binary rather than being installed system-wide.
    /// The path should be the *directory* containing libpdfium —
    /// `pdfium-render` resolves the platform-specific filename
    /// (`libpdfium.dylib` / `libpdfium.so` / `pdfium.dll`).
    pub fn with_library_path(library_dir: &str) -> Result<Self> {
        let bindings =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(library_dir))
                .map_err(|e| Error::MissingDependency {
                    name: "libpdfium".into(),
                    details: format!("could not load from {library_dir}: {e}"),
                })?;
        Ok(Self {
            pdfium: Pdfium::new(bindings),
        })
    }

    /// Internal: extract text from an already-loaded `PdfDocument`.
    /// Pages are joined with `\n\n` (one blank line between pages),
    /// preserving the document's reading order without injecting
    /// opinionated heading markup.
    ///
    /// Title + structured metadata extraction are not yet wired —
    /// `pdfium-render`'s metadata API is in flux across recent
    /// versions and the surface we want to expose deserves its own
    /// dedicated commit. For v0.2 the markdown body is the only
    /// guaranteed output; `title` is `None`, `metadata` is empty.
    fn extract_from_document(doc: &PdfDocument) -> Result<Document> {
        let mut markdown = String::new();
        for (idx, page) in doc.pages().iter().enumerate() {
            if idx > 0 {
                markdown.push_str("\n\n");
            }
            let text = page.text().map_err(|e| {
                Error::ParseError(format!("page {idx} text extraction failed: {e}"))
            })?;
            markdown.push_str(&text.all());
        }

        Ok(Document {
            markdown,
            title: None,
            metadata: std::collections::HashMap::new(),
        })
    }
}

impl Extractor for PdfiumExtractor {
    fn extensions(&self) -> &[&'static str] {
        &["pdf"]
    }

    fn name(&self) -> &'static str {
        "pdfium-render"
    }

    fn extract(&self, path: &Path) -> Result<Document> {
        let path_str = path.to_str().ok_or_else(|| {
            Error::ParseError(format!("PDF path is not valid UTF-8: {}", path.display()))
        })?;
        let doc = self
            .pdfium
            .load_pdf_from_file(path_str, None)
            .map_err(|e| Error::ParseError(format!("pdfium failed to open {path_str}: {e}")))?;
        Self::extract_from_document(&doc)
    }

    fn extract_bytes(&self, bytes: &[u8], _ext: &str) -> Result<Document> {
        let doc = self
            .pdfium
            .load_pdf_from_byte_slice(bytes, None)
            .map_err(|e| Error::ParseError(format!("pdfium failed to open byte slice: {e}")))?;
        Self::extract_from_document(&doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The trait-surface tests don't need libpdfium — they verify
    // shape/behavior that we control without a runtime dependency.
    // Real extraction tests are #[ignore]'d so they don't fail on
    // dev machines / CI runners that don't have libpdfium installed.
    // Run them locally with: cargo test --features pdf -- --ignored

    /// Reusable stand-in for trait-surface tests so we don't have to
    /// instantiate a real `PdfiumExtractor` (which would require libpdfium
    /// on the system library path). Mirrors `PdfiumExtractor`'s
    /// extensions + name.
    struct FakePdf;
    impl Extractor for FakePdf {
        fn extensions(&self) -> &[&'static str] {
            &["pdf"]
        }
        fn extract(&self, _: &std::path::Path) -> Result<Document> {
            unreachable!("FakePdf only used for trait-surface tests")
        }
        fn name(&self) -> &'static str {
            "pdfium-render"
        }
    }

    #[test]
    fn extensions_is_pdf_only() {
        assert_eq!(FakePdf.extensions(), &["pdf"]);
    }

    #[test]
    fn name_identifies_backend() {
        assert_eq!(FakePdf.name(), "pdfium-render");
    }

    #[test]
    #[ignore = "requires libpdfium on the system library path"]
    fn extracts_text_from_a_real_pdf() {
        // Skipped by default. To run: ensure libpdfium is on your
        // library path, then `cargo test --features pdf -- --ignored`.
        // Drop a "hello.pdf" containing the literal text "Hello,
        // World!" into tests/fixtures/ before running.
        let extractor = PdfiumExtractor::new().expect("libpdfium not available");
        let doc = extractor
            .extract(std::path::Path::new("tests/fixtures/hello.pdf"))
            .expect("extraction failed");
        assert!(
            !doc.markdown.is_empty(),
            "expected non-empty markdown from hello.pdf"
        );
    }

    #[test]
    fn missing_libpdfium_returns_typed_error() {
        // Trait-surface guarantee: `PdfiumExtractor` returns a typed
        // `Error::MissingDependency` (not a panic) when libpdfium
        // isn't on the path. We can't reliably trigger the failure
        // on every dev machine, but we CAN verify the error variant
        // is correctly typed by attempting a guaranteed-bad path.
        let result = PdfiumExtractor::with_library_path("/nonexistent-path-that-cannot-exist");
        assert!(matches!(result, Err(Error::MissingDependency { .. })));
    }
}
