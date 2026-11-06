//! Stream the raster font as SVG geometry, without allocation or dependencies.
//!
//! Output is a fragment: wrap it in an SVG element with the SVG namespace and
//! a suitable `viewBox`. Coordinates use the 6×11 source cell; parent SVG
//! transforms supply position, size and rotation. Paint inherits `currentColor`.
//! This module writes paths, not SVG `<text>`, and needs no installed font.
use core::fmt::{self, Write};

use crate::{raster_line, raster_quad};

/// A failed glyph lookup, invalid stroke width, or rejected sink write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// No raster geometry is available. No replacement shape is synthesized.
    MissingGlyph(char),
    /// Stroke width must be finite and greater than zero, in source-cell units.
    InvalidStrokeWidth,
    /// The destination returned `core::fmt::Error`.
    Format,
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        Self::Format
    }
}

enum Geometry {
    Empty,
    Lines(raster_line::GlyphStrips),
    Quads(raster_quad::BoxDrawQuads),
}

fn geometry(character: char) -> Result<Geometry, Error> {
    if character == ' ' {
        Ok(Geometry::Empty)
    } else if let Some(quads) = raster_quad::glyph_quads(character) {
        Ok(Geometry::Quads(quads))
    } else if let Some(strips) = raster_line::glyph_strips(character) {
        Ok(Geometry::Lines(strips))
    } else {
        Err(Error::MissingGlyph(character))
    }
}

fn validate_width(stroke_width: f32) -> Result<(), Error> {
    if stroke_width.is_finite() && stroke_width > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidStrokeWidth)
    }
}

fn write_strips<W: Write + ?Sized>(
    out: &mut W,
    strips: raster_line::GlyphStrips,
    stroke_width: f32,
) -> fmt::Result {
    for strip in strips {
        let first = strip.point(0).unwrap();
        if strip.iter_points().all(|point| point == first) {
            // SVG drops a zero-length polyline with some cap settings. Preserve
            // repeated-point dot records explicitly, including transformed dots.
            write!(
                out,
                "<circle cx=\"{}.5\" cy=\"{}.5\" r=\"{}\" stroke=\"none\"/>",
                first.x(),
                first.y(),
                stroke_width / 2.0
            )?;
        } else {
            out.write_str("<polyline fill=\"none\" points=\"")?;
            for (index, point) in strip.iter_points().enumerate() {
                if index != 0 {
                    out.write_char(' ')?;
                }
                write!(out, "{}.5,{}.5", point.x(), point.y())?;
            }
            out.write_str("\"/>")?;
        }
    }
    Ok(())
}

fn write_geometry<W: Write + ?Sized>(
    out: &mut W,
    glyph: Geometry,
    stroke_width: f32,
) -> fmt::Result {
    match glyph {
        Geometry::Empty => Ok(()),
        Geometry::Lines(strips) => write_strips(out, strips, stroke_width),
        Geometry::Quads(quads) => {
            for quad in quads {
                out.write_str("<polygon stroke=\"none\" points=\"")?;
                for (index, [x, y]) in quad.vertices.into_iter().enumerate() {
                    if index != 0 {
                        out.write_char(' ')?;
                    }
                    write!(out, "{x},{y}")?;
                }
                out.write_str("\"/>")?;
            }
            Ok(())
        }
    }
}

fn begin_group<W: Write + ?Sized>(out: &mut W, stroke_width: f32) -> fmt::Result {
    write!(
        out,
        "<g fill=\"currentColor\" stroke=\"currentColor\" stroke-width=\"{stroke_width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">"
    )
}

/// Write one cell as an SVG `<g>` fragment using raster lines or filled quads.
///
/// Line lattice coordinates receive the renderer's +0.5 pixel-centre offset;
/// quad vertices keep their exact source coordinates. Aliases are transformed
/// before serialization. An ordinary space succeeds with empty geometry.
/// A missing glyph or invalid width leaves `out` untouched; a sink failure can
/// leave partial XML. Nothing from the input is inserted as raw XML or text.
pub fn write_glyph<W: Write + ?Sized>(
    out: &mut W,
    character: char,
    stroke_width: f32,
) -> Result<(), Error> {
    validate_width(stroke_width)?;
    let glyph = geometry(character)?;
    begin_group(out, stroke_width)?;
    write_geometry(out, glyph, stroke_width)?;
    out.write_str("</g>")?;
    Ok(())
}

fn for_each_cell(
    text: &str,
    mut visit: impl FnMut(char) -> Result<(), Error>,
) -> Result<(), Error> {
    for character in text.chars() {
        if character != ' ' {
            if let Some(cells) = crate::special_cells(character) {
                for &cell in cells {
                    visit(cell)?;
                }
                continue;
            }
        }
        visit(character)?;
    }
    Ok(())
}

/// Write a single row of SVG glyph geometry, advancing 6 units for each cell.
///
/// Ordinary spaces are blank cells. Control characters expand to their named
/// labels (for example, LF), and multi-cell icons use their component cells.
/// Unlike `crate::measure_text`, this helper does not display spaces as `SP`.
/// Use an outer SVG group for transforms, color, or additional decoration.
///
/// The complete row is checked before writing: missing glyphs and invalid widths
/// leave the sink untouched. A sink failure may leave partial XML. Text itself
/// is never emitted, so XML metacharacters cannot become markup.
pub fn write_text<W: Write + ?Sized>(
    out: &mut W,
    text: &str,
    stroke_width: f32,
) -> Result<(), Error> {
    validate_width(stroke_width)?;
    for_each_cell(text, |character| geometry(character).map(|_| ()))?;
    begin_group(out, stroke_width)?;
    let mut x = 0usize;
    for_each_cell(text, |character| {
        write!(out, "<g transform=\"translate({x} 0)\">")?;
        write_geometry(out, geometry(character)?, stroke_width)?;
        out.write_str("</g>")?;
        x += crate::FWIDTH;
        Ok(())
    })?;
    out.write_str("</g>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::String;

    #[test]
    fn lines_use_centres_and_aliases_use_transformed_coordinates() {
        let mut left = String::new();
        let mut right = String::new();
        write_glyph(&mut left, '<', 1.25).unwrap();
        write_glyph(&mut right, '>', 1.25).unwrap();
        assert!(left.contains("points=\"4.5,1.5 1.5,4.5 4.5,7.5\""));
        assert!(right.contains("points=\"1.5,1.5 4.5,4.5 1.5,7.5\""));
        assert!(right.contains("stroke-width=\"1.25\""));
    }

    #[test]
    fn repeated_point_dot_becomes_a_visible_circle() {
        let dots = raster_line::decode_strips(&[0xc0, 26, 26]).unwrap();
        let mut out = String::new();
        write_strips(&mut out, dots, 1.5).unwrap();
        assert_eq!(
            out,
            "<circle cx=\"2.5\" cy=\"3.5\" r=\"0.75\" stroke=\"none\"/>"
        );
    }

    #[test]
    fn quads_preserve_cell_edges_and_do_not_receive_line_strokes() {
        let mut out = String::new();
        write_glyph(&mut out, '█', 2.0).unwrap();
        assert!(out.contains("<polygon stroke=\"none\" points=\"0,0 6,0 6,11 0,11\"/>"));
        assert!(!out.contains("polyline"));
    }

    #[test]
    fn missing_glyphs_and_bad_width_do_not_modify_the_sink() {
        let mut out = String::from("prefix");
        assert_eq!(
            write_glyph(&mut out, '\u{10ffff}', 1.0),
            Err(Error::MissingGlyph('\u{10ffff}'))
        );
        assert_eq!(
            write_text(&mut out, "AB\u{10ffff}", 1.0),
            Err(Error::MissingGlyph('\u{10ffff}'))
        );
        for width in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            assert_eq!(
                write_glyph(&mut out, 'A', width),
                Err(Error::InvalidStrokeWidth)
            );
            assert_eq!(
                write_text(&mut out, "A", width),
                Err(Error::InvalidStrokeWidth)
            );
        }
        assert_eq!(out, "prefix");
    }

    #[test]
    fn spaces_advance_without_shapes_and_xml_metacharacters_are_geometry() {
        let mut out = String::new();
        write_text(&mut out, "<&> ", 1.0).unwrap();
        assert_eq!(out.matches("transform=\"translate(").count(), 4);
        assert!(out.contains("<g transform=\"translate(18 0)\"></g>"));
        assert!(!out.contains("<&>"));
        assert!(!out.contains("<text"));
        assert!(!out.contains('&'));
        let mut space = String::new();
        write_glyph(&mut space, ' ', 1.0).unwrap();
        assert!(!space.contains("polyline"));
        assert!(!space.contains("polygon"));
    }

    #[test]
    fn control_labels_use_the_same_cell_geometry_as_their_letters() {
        let mut control = String::new();
        let mut label = String::new();
        write_text(&mut control, "\n", 1.0).unwrap();
        write_text(&mut label, "LF", 1.0).unwrap();
        assert_eq!(control, label);
    }

    #[test]
    fn sink_failures_are_reported() {
        struct Reject;
        impl Write for Reject {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                Err(fmt::Error)
            }
        }
        assert_eq!(write_glyph(&mut Reject, 'A', 1.0), Err(Error::Format));
        assert_eq!(write_text(&mut Reject, "A", 1.0), Err(Error::Format));
    }
}
