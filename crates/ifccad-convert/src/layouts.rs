//! Interpretation of CAD paper coordinates and physical sheet media.
use crate::{diagnostics::diagnostic, IfccadConversionError, IfccadDiagnostic, IfccadMappings};
use num_rational::BigRational;
use ocdraw::ifccad::{IfccadPaperLayout, IfccadPaperSize};
use opencadcodec::objects::{Layout, ObjectType};
use opencadcodec::{CadDocument, EntityType, Handle};

pub(crate) fn from_cad(
    layout: &Layout,
    issues: &mut Vec<IfccadDiagnostic>,
) -> (String, Option<IfccadPaperSize>) {
    let location = format!("layout/{}", layout.name);
    let paper = if layout.paper_width == 0. && layout.paper_height == 0. {
        None
    } else if layout.paper_width.is_finite()
        && layout.paper_width > 0.
        && layout.paper_height.is_finite()
        && layout.paper_height > 0.
    {
        Some(IfccadPaperSize {
            width: layout.paper_width,
            height: layout.paper_height,
            length_unit: "mm".into(),
        })
    } else {
        issues.push(diagnostic(
            "paper-medium",
            format!("{location}.paper"),
            "invalid or incomplete CAD medium omitted; layout and geometry retained",
        ));
        None
    };
    let fixed = layout.plot_scale_type == 1
        && !layout.plot_flags.use_standard_scale
        && layout.plot_scale_numerator == 1.
        && layout.plot_scale_denominator == 1.
        && layout.plot_scale_factor == 1.
        && matches!(layout.plot_paper_units, 0 | 1);
    let default = Layout::new(&layout.name);
    let unconfigured = layout.paper_width == 0.
        && layout.paper_height == 0.
        && layout.plot_scale_type == default.plot_scale_type
        && layout.plot_scale_numerator == default.plot_scale_numerator
        && layout.plot_scale_denominator == default.plot_scale_denominator
        && layout.plot_scale_factor == default.plot_scale_factor
        && layout.plot_paper_units == default.plot_paper_units
        && layout.plot_flags.use_standard_scale == default.plot_flags.use_standard_scale;
    let unit = if fixed {
        if layout.plot_paper_units == 0 {
            "in"
        } else {
            "mm"
        }
    } else {
        if !unconfigured {
            issues.push(diagnostic("paper-coordinate-unit",format!("{location}.plotMapping"),"physical Paper coordinate unit not established by supported fixed 1:1 mapping; numeric coordinates retained as unitless"));
        }
        "unitless"
    };
    (unit.into(), paper)
}

fn media_mm(
    paper: &IfccadPaperSize,
    location: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<(f64, f64)> {
    // Exact registry factors in millimetres. A parsec is not a rational SI factor.
    let (numerator, denominator): (&str, &str) = match paper.length_unit.as_str() {
        "mm" => ("1", "1"),
        "in" => ("127", "5"),
        "cm" => ("10", "1"),
        "m" => ("1000", "1"),
        "ft" => ("1524", "5"),
        "yd" => ("4572", "5"),
        "mi" => ("1609344", "1"),
        "km" => ("1000000", "1"),
        "microin" => ("127", "5000000"),
        "mil" => ("127", "5000"),
        "angstrom" => ("1", "10000000"),
        "nm" => ("1", "1000000"),
        "um" => ("1", "1000"),
        "dm" => ("100", "1"),
        "dam" => ("10000", "1"),
        "hm" => ("100000", "1"),
        "Gm" => ("1000000000000", "1"),
        "au" => ("149597870700000", "1"),
        "ly" => ("9460730472580800000", "1"),
        "usSurveyFoot" => ("1200000", "3937"),
        "usSurveyInch" => ("100000", "3937"),
        "usSurveyYard" => ("3600000", "3937"),
        "usSurveyMile" => ("6336000000", "3937"),
        _ => {
            issues.push(diagnostic(
                "paper-medium",
                location,
                "physical medium unit has no supported exact millimetre mapping; medium omitted",
            ));
            return None;
        }
    };
    let factor = BigRational::new(
        numerator.parse().expect("static unit numerator"),
        denominator.parse().expect("static unit denominator"),
    );
    let float_factor = numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();
    let mut mapped = Vec::new();
    for value in [paper.width, paper.height] {
        let result = value * float_factor;
        let exact = BigRational::from_float(value).map(|v| v * &factor);
        if exact.is_none() || exact != BigRational::from_float(result) || result <= 0. {
            issues.push(diagnostic(
                "rounding",
                location,
                "physical medium conversion is not an exact positive finite binary64 value",
            ));
            return None;
        }
        mapped.push(result);
    }
    Some((mapped[0], mapped[1]))
}

pub(crate) fn allocate(
    document: &mut CadDocument,
    papers: &[IfccadPaperLayout],
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Vec<(u64, Handle)>, IfccadConversionError> {
    // Check before constructing any target so native u32 tabs never truncate to i16.
    let mut names = std::collections::BTreeSet::from(["MODEL".to_string()]);
    for paper in papers {
        if !names.insert(paper.name.to_uppercase()) {
            return Err(IfccadConversionError::CadConstruction(
                "ambiguous CAD layout name".into(),
            ));
        }
        i16::try_from(paper.tab_index).map_err(|_| {
            IfccadConversionError::CadConstruction("layout tab index exceeds CAD i16 range".into())
        })?;
    }
    let mut ordered: Vec<_> = papers.iter().collect();
    ordered.sort_by_key(|p| p.tab_index);
    let mut owners = Vec::new();
    for paper in ordered {
        let loc = format!("layout/{}", paper.id);
        if !matches!(paper.length_unit.as_str(), "unitless" | "in" | "mm") {
            issues.push(diagnostic(
                "paper-coordinate-unit",
                &loc,
                "Paper coordinate unit needs deferred geometry/unit mapping; whole layout omitted",
            ));
            for entity in &paper.entities {
                issues.push(diagnostic(
                    "entity-skipped",
                    format!("entity/{}", entity.id),
                    "entity omitted with its unsupported Paper coordinate unit",
                ));
            }
            continue;
        }
        let handle = if owners.is_empty() {
            let Some((handle, old_name)) = document.objects.iter().find_map(|(h, o)| match o {
                ObjectType::Layout(l)
                    if l.block_record == document.header.paper_space_block_handle =>
                {
                    Some((*h, l.name.clone()))
                }
                _ => None,
            }) else {
                return Err(IfccadConversionError::CadConstruction(
                    "missing runtime Paper scaffold".into(),
                ));
            };
            if let Some(ObjectType::Dictionary(dictionary)) = document
                .objects
                .get_mut(&document.header.acad_layout_dict_handle)
            {
                for (name, target) in &mut dictionary.entries {
                    if *target == handle && *name == old_name {
                        *name = paper.name.clone();
                    }
                }
            }
            let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
                unreachable!()
            };
            layout.name = paper.name.clone();
            let mut viewport = opencadcodec::entities::Viewport::new();
            viewport.id = 1;
            document
                .add_entity_to_layout(EntityType::Viewport(viewport), &paper.name)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
            handle
        } else {
            document
                .add_layout(&paper.name)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?
        };
        let media = paper
            .paper
            .as_ref()
            .and_then(|m| media_mm(m, &format!("{loc}.paper"), issues));
        let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
            unreachable!()
        };
        layout.tab_order = i16::try_from(paper.tab_index).expect("checked range");
        if let Some((width, height)) = media {
            layout.paper_width = width;
            layout.paper_height = height;
        }
        if paper.length_unit != "unitless" {
            layout.plot_paper_units = if paper.length_unit == "in" { 0 } else { 1 };
            layout.plot_scale_type = 1;
            layout.plot_flags.use_standard_scale = false;
            layout.plot_scale_numerator = 1.;
            layout.plot_scale_denominator = 1.;
            layout.plot_scale_factor = 1.;
        }
        let owner = layout.block_record;
        // DWG implicit Paper mode omits a marker's owner. Unlike drawable
        // membership, the reader cannot recover that owner from the record's
        // ordered contents. Explicit owned markers retain every sheet boundary.
        let record = document
            .block_records
            .iter()
            .find(|b| b.handle == owner)
            .expect("allocated Paper record");
        let mut begin = opencadcodec::entities::Block::new(&record.name, record.base_point);
        begin.common.handle = record.block_entity_handle;
        begin.common.owner_handle = owner;
        begin.common.entity_mode = Some(0);
        let mut end = opencadcodec::entities::BlockEnd::new();
        end.common.handle = record.block_end_handle;
        end.common.owner_handle = owner;
        end.common.entity_mode = Some(0);
        for marker in [EntityType::Block(begin), EntityType::BlockEnd(end)] {
            document
                .add_entity(marker)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
        }
        owners.push((paper.id, owner));
        mappings.layouts.insert(paper.id, handle);
    }
    Ok(owners)
}
