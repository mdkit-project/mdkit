//! Extract a document to markdown.
//!
//! Usage:
//!
//! ```bash
//! cargo run --example extract -- /path/to/document.pdf
//! ```
//!
//! With the default features (in-process Rust backends only):
//! handles PDF (libpdfium required at runtime), HTML, IPYNB, plus
//! tabular formats via the calamine + csv readers (registered but
//! rarely the primary use case for "extract to markdown").
//!
//! For full coverage including DOCX / PPTX / EPUB / RTF / ODT /
//! LaTeX (Pandoc) and OCR (macOS Vision / `Windows.Media.Ocr` /
//! ONNX `PaddleOCR`), enable the relevant features:
//!
//! ```bash
//! cargo run --example extract --features pandoc -- /path/to/report.docx
//! cargo run --example extract --features ocr-platform -- /path/to/scan.pdf
//! ```
//!
//! On startup the example logs which backends registered cleanly
//! and which fell through (e.g. libpdfium not on the library
//! search path) so missing runtime deps are debuggable without
//! reading mdkit source.

use std::env;
use std::path::Path;
use std::process::ExitCode;

use mdkit::Engine;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: extract <path-to-document>");
        return ExitCode::FAILURE;
    };

    // `with_defaults_diagnostic` returns the engine plus a list of
    // backends that failed to register, so we can surface "PDF
    // disabled because libpdfium isn't on the search path" rather
    // than silently degrading.
    let (engine, errors) = Engine::with_defaults_diagnostic();
    for (backend, err) in &errors {
        eprintln!("[mdkit] backend `{backend}` not registered: {err}");
    }
    if errors.is_empty() {
        eprintln!("[mdkit] all backends registered cleanly");
    }
    eprintln!();

    match engine.extract(Path::new(&path)) {
        Ok(doc) => {
            print_doc(&doc);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_doc(doc: &mdkit::Document) {
    if let Some(title) = &doc.title {
        println!("# {title}");
        println!();
    }

    if !doc.metadata.is_empty() {
        // Surface non-trivial metadata at the top so callers can
        // see, for example, that a scanned PDF went through the
        // OCR fallback (`extractor_chain: pdfium-render →
        // vision-macos`).
        let mut keys: Vec<&String> = doc.metadata.keys().collect();
        keys.sort();
        for key in keys {
            println!("<!-- {key}: {} -->", doc.metadata[key]);
        }
        println!();
    }

    print!("{}", doc.markdown);
    if !doc.markdown.ends_with('\n') {
        println!();
    }
}
