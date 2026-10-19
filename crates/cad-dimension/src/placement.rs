use crate::*;
use cad_geometry_convert::{
    geometry::{
        blocks::PairedPoint,
        cad_plane,
        numeric::{exact, round_nearest},
        stored_normal,
    },
    GeometryPair,
};
use ifccad::{
    dimension::*,
    geometry_kernel::{CoordinateFrame3, Point3, Vector3 as V},
    text::{MTextAttachment, TextRun},
};
use num_rational::BigRational as Q;
use num_traits::{ToPrimitive, Zero};
use opencadcodec::{
    entities::{AttachmentPointType, Dimension, DimensionAligned, DimensionLinear, DimensionType},
    CadDocument, Handle, Vector3,
};

pub struct PreparedCadDimension {
    pub definition: DimensionDefinition,
    pub placement: CoordinateFrame3,
    pub text_placement: DimensionTextPlacement,
    pub label: DimensionLabel,
    pub style_handle: Handle,
    pub overrides: CadDimensionStyleOverrides,
    pub pairs: Vec<GeometryPair>,
    pub losses: Vec<CadDimensionLoss>,
    pub residual_xdata: Vec<opencadcodec::xdata::ExtendedDataRecord>,
    pub pending_associations: Vec<CadDimensionPointReference>,
    pub consumed_association_objects: Vec<Handle>,
}
pub struct PreparedDimensionTarget {
    pub entity: Dimension,
    pub pairs: Vec<GeometryPair>,
    pub losses: Vec<CadDimensionLoss>,
}
fn xyz(v: Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn cv(v: [f64; 3]) -> Vector3 {
    Vector3::new(v[0], v[1], v[2])
}
fn check(field: &str, v: [f64; 3]) -> Result<(), CadDimensionPreparationError> {
    if v.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(invalid(field, "nonfinite active point"))
    }
}
fn round(v: &Q) -> Result<f64, CadDimensionPreparationError> {
    round_nearest(v).map_err(|_| numeric("placement", "finite range or projection failure"))
}
fn dot(a: &[Q; 3], b: [f64; 3]) -> Q {
    (0..3).map(|i| &a[i] * exact(b[i])).sum()
}
fn frame_vectors(p: CoordinateFrame3) -> ([f64; 3], [f64; 3], [f64; 3]) {
    (
        p.origin().components(),
        p.x_axis().components(),
        p.y_axis().components(),
    )
}
fn world(p: CoordinateFrame3, local: [f64; 2]) -> [Q; 3] {
    let (o, u, v) = frame_vectors(p);
    std::array::from_fn(|i| {
        exact(o[i]) + exact(u[i]) * exact(local[0]) + exact(v[i]) * exact(local[1])
    })
}
fn project(
    p: CoordinateFrame3,
    position: [Q; 3],
) -> Result<[f64; 2], CadDimensionPreparationError> {
    let (o, u, v) = frame_vectors(p);
    let d = std::array::from_fn(|i| position[i].clone() - exact(o[i]));
    let a = dot(&u.map(exact), u);
    let b = dot(&u.map(exact), v);
    let c = dot(&v.map(exact), v);
    let determinant = &a * &c - &b * &b;
    if determinant <= Q::zero() {
        return Err(numeric("placement", "singular plane"));
    }
    let x = dot(&d, u);
    let y = dot(&d, v);
    Ok([
        round(&((&x * &c - &y * &b) / &determinant))?,
        round(&((y * a - x * b) / determinant))?,
    ])
}
fn pair(source: [Q; 3], target: [Q; 3]) -> GeometryPair {
    let p = PairedPoint::new(source, target);
    let d2 = p.squared_deviation().1;
    GeometryPair {
        points: vec![(p, d2)],
        curves: Vec::new(),
    }
}
fn attachment(v: AttachmentPointType) -> MTextAttachment {
    match v {
        AttachmentPointType::TopLeft => MTextAttachment::TopLeft,
        AttachmentPointType::TopCenter => MTextAttachment::TopCenter,
        AttachmentPointType::TopRight => MTextAttachment::TopRight,
        AttachmentPointType::MiddleLeft => MTextAttachment::MiddleLeft,
        AttachmentPointType::MiddleCenter => MTextAttachment::MiddleCenter,
        AttachmentPointType::MiddleRight => MTextAttachment::MiddleRight,
        AttachmentPointType::BottomLeft => MTextAttachment::BottomLeft,
        AttachmentPointType::BottomCenter => MTextAttachment::BottomCenter,
        AttachmentPointType::BottomRight => MTextAttachment::BottomRight,
    }
}
fn cad_attachment(v: MTextAttachment) -> AttachmentPointType {
    match v {
        MTextAttachment::TopLeft => AttachmentPointType::TopLeft,
        MTextAttachment::TopCenter => AttachmentPointType::TopCenter,
        MTextAttachment::TopRight => AttachmentPointType::TopRight,
        MTextAttachment::MiddleLeft => AttachmentPointType::MiddleLeft,
        MTextAttachment::MiddleCenter => AttachmentPointType::MiddleCenter,
        MTextAttachment::MiddleRight => AttachmentPointType::MiddleRight,
        MTextAttachment::BottomLeft => AttachmentPointType::BottomLeft,
        MTextAttachment::BottomCenter => AttachmentPointType::BottomCenter,
        MTextAttachment::BottomRight => AttachmentPointType::BottomRight,
    }
}
fn label_from(value: Option<&str>) -> Result<DimensionLabel, CadDimensionPreparationError> {
    let Some(value) = value else {
        return Ok(DimensionLabel::Generated);
    };
    if value.is_empty() || value == "<>" {
        return Ok(DimensionLabel::Generated);
    }
    if value == " " {
        return Ok(DimensionLabel::Suppressed);
    }
    if crate::styles::unqualified_text(value) {
        return Err(unsupported("text", "unqualified markup or dynamic field"));
    }
    let mut parts = Vec::new();
    let mut remaining = value;
    while let Some((prefix, tail)) = remaining.split_once("<>") {
        if !prefix.is_empty() {
            parts.push(DimensionLabelPart::Literal(vec![TextRun {
                text: prefix.into(),
                ..Default::default()
            }]));
        }
        parts.push(DimensionLabelPart::Measurement);
        remaining = tail;
    }
    if !remaining.is_empty() {
        parts.push(DimensionLabelPart::Literal(vec![TextRun {
            text: remaining.into(),
            ..Default::default()
        }]));
    }
    Ok(DimensionLabel::Template(parts))
}
fn label_to(v: &DimensionLabel) -> Result<Option<String>, CadDimensionPreparationError> {
    match v {
        DimensionLabel::Generated => Ok(None),
        DimensionLabel::Suppressed => Ok(Some(" ".into())),
        DimensionLabel::Template(parts) => {
            let mut text = String::new();
            for p in parts {
                match p {
                    DimensionLabelPart::Measurement => text.push_str("<>"),
                    DimensionLabelPart::Literal(runs) => {
                        for r in runs {
                            if r.underline
                                || r.overline
                                || r.strike_through
                                || r.text.contains("<>")
                                || crate::styles::unqualified_text(&r.text)
                            {
                                return Err(unsupported(
                                    "label",
                                    "unqualified decorated or token-ambiguous literal",
                                ));
                            }
                            text.push_str(&r.text);
                        }
                    }
                }
            }
            if text.is_empty() || text == " " || text == "<>" {
                return Err(unsupported(
                    "label",
                    "CAD sentinel cannot preserve this template choice",
                ));
            }
            Ok(Some(text))
        }
    }
}
pub fn prepare_dimension_from_cad(
    source: &Dimension,
    doc: &CadDocument,
) -> Result<PreparedCadDimension, CadDimensionPreparationError> {
    let base = source.base();
    let (first, second, line, rotation, extension) = match source {
        Dimension::Linear(d) => {
            if base.dimension_type != DimensionType::Linear {
                return Err(invalid("type", "conflicting Dimension kind"));
            }
            (
                d.first_point,
                d.second_point,
                d.definition_point,
                Some(d.rotation),
                d.ext_line_rotation,
            )
        }
        Dimension::Aligned(d) => {
            if base.dimension_type != DimensionType::Aligned {
                return Err(invalid("type", "conflicting Dimension kind"));
            }
            (
                d.first_point,
                d.second_point,
                d.definition_point,
                None,
                d.ext_line_rotation,
            )
        }
        _ => {
            return Err(unsupported(
                "kind",
                "only linear and aligned definitions are qualified in slice 1",
            ))
        }
    };
    for (field, p) in [
        ("firstPoint", first),
        ("secondPoint", second),
        ("dimensionLinePoint", line),
    ] {
        check(field, xyz(p))?;
    }
    if !extension.is_finite()
        || rotation.is_some_and(|v| !v.is_finite())
        || !base.text_rotation.is_finite()
        || !base.horizontal_direction.is_finite()
    {
        return Err(invalid("angles", "nonfinite angle"));
    }
    if rotation.is_none() && extension != 0.0 {
        return Err(unsupported(
            "extensionDirectionAngle",
            "aligned oblique extension is not qualified by the DXF linear group-52 rule",
        ));
    }
    if base.horizontal_direction != 0.0 {
        return Err(unsupported(
            "horizontalDirection",
            "UCS text-direction reparameterization needs a separate qualifier",
        ));
    }
    if base.insertion_scale != Vector3::new(1.0, 1.0, 1.0)
        || base.insertion_rotation != 0.0
        || base.insertion_point != Vector3::ZERO
    {
        return Err(unsupported("insertion", "cloned picture transforms"));
    }
    if base
        .common
        .extended_data
        .records()
        .iter()
        .any(|r| r.application_name.eq_ignore_ascii_case("AcadAnnotative"))
    {
        return Err(unsupported(
            "annotative",
            "annotation context is a later slice",
        ));
    }
    let axis = cad_plane(base.normal).ok_or_else(|| invalid("normal", "invalid plane normal"))?;
    let plane = CoordinateFrame3::try_new(
        Point3::new(first.x, first.y, first.z),
        V::new(axis.u[0], axis.u[1], axis.u[2]),
        V::new(axis.v[0], axis.v[1], axis.v[2]),
    )
    .map_err(|e| numeric("placement", &e.to_string()))?;
    let a = [0.0, 0.0];
    let b = project(plane, xyz(second).map(exact))?;
    let line_local = project(plane, xyz(line).map(exact))?;
    let definition = if let Some(angle) = rotation {
        DimensionDefinition::Linear {
            first_point: a,
            second_point: b,
            direction_angle: angle,
            dimension_line_point: line_local,
            extension_direction_angle: if extension == 0.0 {
                None
            } else {
                Some(angle + extension)
            },
        }
    } else {
        DimensionDefinition::Aligned {
            first_point: a,
            second_point: b,
            dimension_line_point: line_local,
            extension_direction_angle: None,
        }
    };
    let DimensionMeasurement::Distance(measurement) =
        evaluate_dimension(&definition).map_err(|e| numeric("measurement", &e.to_string()))?
    else {
        unreachable!()
    };
    let mut pairs = vec![
        pair(xyz(first).map(exact), world(plane, a)),
        pair(xyz(second).map(exact), world(plane, b)),
        pair(xyz(line).map(exact), world(plane, line_local)),
    ];
    let style = doc
        .dim_styles
        .get(&base.style_name)
        .ok_or_else(|| invalid("styleName", "unresolved DimensionStyle"))?;
    let parsed = parse_dimension_overrides(base.common.extended_data.records())?;
    let mut parsed = resolve_dimension_overrides(parsed, style, doc)?;
    parsed.values.arrows.flip_first = Some(base.flip_arrow1);
    parsed.values.arrows.flip_second = Some(base.flip_arrow2);
    parsed.values.text.line_spacing = Some(match base.line_spacing_style {
        1 => DimensionLineSpacing::AtLeast {
            factor: base.line_spacing_factor,
        },
        2 => DimensionLineSpacing::Exact {
            factor: base.line_spacing_factor,
        },
        _ => return Err(unsupported("lineSpacingStyle", "unknown line spacing")),
    });
    let text_placement = if base.text_user_positioned {
        check("textMiddlePoint", xyz(base.text_middle_point))?;
        let raw = xyz(base.text_middle_point);
        let source_q = std::array::from_fn(|i| {
            exact(axis.u[i]) * exact(raw[0])
                + exact(axis.v[i]) * exact(raw[1])
                + exact(axis.n[i]) * exact(raw[2])
        });
        let position = project(plane, source_q.clone())?;
        pairs.push(pair(source_q, world(plane, position)));
        DimensionTextPlacement::Explicit {
            position,
            rotation: base.text_rotation,
            attachment: attachment(base.attachment_point),
        }
    } else {
        if base.text_rotation != 0.0 {
            return Err(unsupported(
                "textRotation",
                "rotated automatic text position",
            ));
        }
        DimensionTextPlacement::StyleDriven
    };
    let mut effective = parsed.effective.clone();
    parsed.values.apply(&mut effective);
    validate_dimension_settings(&effective).map_err(|e| invalid(&e.field, &e.to_string()))?;
    let mut losses = parsed.losses;
    if stored_normal(plane) != Some(base.normal) {
        losses.push(CadDimensionLoss {
            field: "normal".into(),
            detail: "source normal normalized in native plane preparation".into(),
        });
    }
    if base.actual_measurement != measurement {
        losses.push(CadDimensionLoss{field:"actualMeasurement".into(),detail:"CAD measurement cache differs from the stored definition; cache omitted, definition retained".into()});
    }
    if !base.block_name.is_empty() {
        losses.push(CadDimensionLoss{field:"pictureBlock".into(),detail:"native definition has no CAD picture-cache link; ordinary block contents remain independently audited".into()});
    }
    if base.dwg_unknown_bit || base.dwg_flags_byte & !1 != 0 {
        losses.push(CadDimensionLoss {
            field: "dwgFlags".into(),
            detail: "unqualified source bookkeeping/editing bits omitted".into(),
        });
    }
    if base.definition_point != line {
        losses.push(CadDimensionLoss{field:"base/definitionPoint".into(),detail:"redundant CAD base point differs from authoritative variant point; base copy omitted".into()});
    }
    if base.version != 0 {
        losses.push(CadDimensionLoss {
            field: "version".into(),
            detail: "CAD version bookkeeping omitted".into(),
        });
    }
    if !base.text_user_positioned {
        losses.push(CadDimensionLoss{field:"textMiddlePoint/attachmentPoint".into(),detail:"automatic display anchor and cache attachment omitted; native text retains DIMTAD style placement".into()});
    }
    let mut associations = doc
        .objects
        .iter()
        .filter_map(|(h, o)| {
            if let opencadcodec::objects::ObjectType::Associative(o) = o {
                if let opencadcodec::objects::AssociativeData::DimensionAssociation(a) = &o.data {
                    if a.dimension == base.common.handle && !a.dimension.is_null() {
                        return Some((*h, a));
                    }
                }
                None
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    associations.sort_by_key(|(h, _)| *h);
    if associations.len() > 1 {
        return Err(invalid(
            "associations",
            "multiple DIMASSOC objects claim one dimension",
        ));
    }
    let mut pending_associations = Vec::new();
    let mut consumed_association_objects = Vec::new();
    for (h, a) in associations {
        match crate::prepare_point_references(a, doc) {
            Ok(refs) => {
                if refs.iter().any(|r| {
                    doc.get_entity(r.source)
                        .is_some_and(|e| e.common().owner_handle != base.common.owner_handle)
                }) {
                    losses.push(CadDimensionLoss {
                        field: format!("associations/{h}"),
                        detail:
                            "cross-owner relation detached; stored dimension definition retained"
                                .into(),
                    });
                } else {
                    pending_associations = refs;
                    consumed_association_objects.push(h);
                    losses.push(CadDimensionLoss{field:format!("associations/{h}/cadState"),detail:"cached snap points and CAD object envelope/editing state omitted; typed source/selector relation retained".into()});
                }
            }
            Err(CadDimensionPreparationError::Unsupported { field, detail }) => {
                losses.push(CadDimensionLoss {
                    field: format!("associations/{h}/{field}"),
                    detail: format!("{detail}; relation detached, stored definition retained"),
                })
            }
            Err(e) => return Err(e),
        }
    }
    Ok(PreparedCadDimension {
        definition,
        placement: plane,
        text_placement,
        label: label_from(base.text_override())?,
        style_handle: style.handle,
        overrides: parsed.values,
        pairs,
        losses,
        residual_xdata: parsed.residual,
        pending_associations,
        consumed_association_objects,
    })
}
pub fn prepare_dimension_to_cad(
    definition: &DimensionDefinition,
    placement: CoordinateFrame3,
    text: &DimensionTextPlacement,
    label: &DimensionLabel,
) -> Result<PreparedDimensionTarget, CadDimensionPreparationError> {
    validate_dimension_definition(definition).map_err(|e| invalid(&e.field, &e.to_string()))?;
    validate_dimension_text_placement(text).map_err(|e| invalid(&e.field, &e.to_string()))?;
    let normal = stored_normal(placement)
        .ok_or_else(|| numeric("normal", "unrepresentable target normal"))?;
    let axis = cad_plane(normal).ok_or_else(|| numeric("normal", "invalid target basis"))?;
    let (_, u, _) = frame_vectors(placement);
    let phase = fpmath::atan2(
        dot(&u.map(exact), axis.v)
            .to_f64()
            .ok_or_else(|| numeric("phase", "range"))?,
        dot(&u.map(exact), axis.u)
            .to_f64()
            .ok_or_else(|| numeric("phase", "range"))?,
    );
    let (a, b) = definition.points();
    let a_q = world(placement, a);
    let b_q = world(placement, b);
    let a_c = [round(&a_q[0])?, round(&a_q[1])?, round(&a_q[2])?];
    let b_c = [round(&b_q[0])?, round(&b_q[1])?, round(&b_q[2])?];
    let (line, extension) = match definition {
        DimensionDefinition::Linear {
            dimension_line_point,
            extension_direction_angle,
            ..
        }
        | DimensionDefinition::Aligned {
            dimension_line_point,
            extension_direction_angle,
            ..
        } => (*dimension_line_point, *extension_direction_angle),
    };
    let extension_offset = match definition {
        DimensionDefinition::Linear {
            direction_angle, ..
        } => {
            if let Some(v) = extension {
                let delta = v - *direction_angle;
                if !delta.is_finite() {
                    return Err(numeric(
                        "extensionDirectionAngle",
                        "target angle offset overflow",
                    ));
                }
                if delta.abs() <= 1e-12 {
                    return Err(unsupported(
                        "extensionDirectionAngle",
                        "pinned DXF omits an explicit near-zero oblique offset",
                    ));
                }
                delta
            } else {
                0.0
            }
        }
        DimensionDefinition::Aligned { .. } => {
            if extension.is_some() {
                return Err(unsupported(
                    "extensionDirectionAngle",
                    "aligned oblique extension transport is unqualified",
                ));
            }
            0.0
        }
    };
    let line_q = world(placement, line);
    let line_c = [round(&line_q[0])?, round(&line_q[1])?, round(&line_q[2])?];
    let mut entity = match definition {
        DimensionDefinition::Linear {
            direction_angle, ..
        } => {
            let mut d = DimensionLinear::new(cv(a_c), cv(b_c));
            d.definition_point = cv(line_c);
            d.rotation = *direction_angle + phase;
            d.ext_line_rotation = extension_offset;
            Dimension::Linear(d)
        }
        DimensionDefinition::Aligned { .. } => {
            let mut d = DimensionAligned::new(cv(a_c), cv(b_c));
            d.definition_point = cv(line_c);
            Dimension::Aligned(d)
        }
    };
    let base = entity.base_mut();
    base.definition_point = cv(line_c);
    base.normal = normal;
    base.horizontal_direction = -phase;
    let DimensionMeasurement::Distance(measurement) =
        evaluate_dimension(definition).map_err(|e| numeric("measurement", &e.to_string()))?
    else {
        unreachable!()
    };
    base.actual_measurement = measurement;
    base.set_text_override(label_to(label)?);
    let mut pairs = vec![
        pair(a_q, a_c.map(exact)),
        pair(b_q, b_c.map(exact)),
        pair(line_q, line_c.map(exact)),
    ];
    if let DimensionTextPlacement::Explicit {
        position,
        rotation,
        attachment,
    } = text
    {
        let source_q = world(placement, *position);
        let local = [
            round(&dot(&source_q, axis.u))?,
            round(&dot(&source_q, axis.v))?,
            round(&dot(&source_q, axis.n))?,
        ];
        base.text_middle_point = cv(local);
        base.text_user_positioned = true;
        base.text_rotation = *rotation;
        base.attachment_point = cad_attachment(*attachment);
        let target_q = std::array::from_fn(|i| {
            exact(axis.u[i]) * exact(local[0])
                + exact(axis.v[i]) * exact(local[1])
                + exact(axis.n[i]) * exact(local[2])
        });
        pairs.push(pair(source_q, target_q));
    }
    let mut losses=vec![CadDimensionLoss{field:"pictureBlock".into(),detail:"target has no generated anonymous picture block; visibility requires CAD-host regeneration, unqualified by this codec".into()}];
    if phase != 0.0 {
        losses.push(CadDimensionLoss{field:"parameterization".into(),detail:"native in-plane basis reparameterized to CAD OCS; UCS text-direction transport is not qualified for reimport".into()});
    }
    Ok(PreparedDimensionTarget {
        entity,
        pairs,
        losses,
    })
}
