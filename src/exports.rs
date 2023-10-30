use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use typst::diag::StrResult;
use typst::doc::Document;
use typst::geom::Color;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OutputFormat {
    Pdf,
    Png,
    Svg,
}
/// Export to a PDF.
pub fn export_pdf(document: &Document) -> StrResult<String> {
    let pdf_buffer = typst::export::pdf(document);
    let pdf_base64: String = base64_encode(pdf_buffer);

    StrResult::Ok(pdf_base64)
}

fn base64_encode(buffer: Vec<u8>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(buffer)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ImageExportFormat {
    Png,
    Svg,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageData {
    name: String,
    type_: ImageExportFormat,
    pages: Vec<PageData>,
}
#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    number: usize,
    data: String,
}
impl PageData {
    pub fn new(number: usize, data: String) -> StrResult<Self> {
        Ok(Self {
            number: number,
            data: data,
        })
    }
}

impl ImageData {
    pub fn new(name: String, type_: ImageExportFormat) -> StrResult<Self> {
        Ok(Self {
            name: name,
            type_: type_,
            pages: Vec::new(),
        })
    }
}

/// Export to one or multiple PNGs.
pub fn export_image(
    document: &Document,
    fmt: ImageExportFormat,
    ppi: f32, //144.0
) -> StrResult<String> {
    // Determine whether we have a `{n}` numbering.
    // let output = command.output();
    // let string = output.to_str().unwrap_or_default();
    // let numbered = string.contains("{n}");
    // if !numbered && document.pages.len() > 1 {
    //     bail!("cannot export multiple images without `{{n}}` in output path");
    // }

    // Find a number width that accommodates all pages. For instance, the
    // first page should be numbered "001" if there are between 100 and
    // 999 pages.

    // let width = 1 + document.pages.len().checked_ilog10().unwrap_or(0) as usize;
    // let mut storage: Vec<String> = !Vec[..];
    let mut image_data: ImageData =
        ImageData::new("imageDate".to_string(), ImageExportFormat::Png).unwrap();

    for (i, frame) in document.pages.iter().enumerate() {
        match fmt {
            ImageExportFormat::Png => {
                let pixmap = typst::export::render(frame, ppi / 72.0, Color::WHITE);

                let png_buffer = pixmap.encode_png().unwrap();
                let png_base64 = base64_encode(png_buffer);
                image_data.pages.push(PageData {
                    number: i,
                    data: png_base64.to_string(),
                })
            }

            ImageExportFormat::Svg => {
                let svg = typst::export::svg(frame);
                let svg_base64 = base64_encode(svg.into_bytes());
                image_data.pages.push(PageData {
                    number: i,
                    data: svg_base64.to_string(),
                })
            }
        }
    }

    // let imageData = !json({

    // });

    let json_string_data = serde_json::to_string(&image_data).unwrap();

    StrResult::Ok(json_string_data)
}
