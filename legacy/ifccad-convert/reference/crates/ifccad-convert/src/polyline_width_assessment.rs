//! Authored width state and actual committed segment-band certificates.
use crate::*;
use ifccad::{
    geometry_kernel::{self, PolylineWidthBoundsReason as Reason},
    *,
};
use opencadcodec::{CadDocument, EntityType};
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPolylineWidthBandCoverage {
    ZeroWidth,
    Proven,
    Unassessed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPolylineWidthSurfaceCoverage {
    Proven,
    Unassessed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPolylineWidthDisplayCoverage {
    Unassessed,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadPolylineWidthSegmentAssessment {
    pub segment_index: usize,
    pub band_coverage: IfccadPolylineWidthBandCoverage,
    pub deviation: Option<IfccadDistanceInterval>,
    pub reason: Option<Reason>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadPolylineWidthAssessment {
    pub source: IfccadGeometryEntitySource,
    pub widths: IfccadPlanarPolylineWidths,
    pub effective_widths: Vec<[f64; 2]>,
    pub segments: Vec<IfccadPolylineWidthSegmentAssessment>,
    pub surface_coverage: IfccadPolylineWidthSurfaceCoverage,
    pub surface_reasons: Vec<Reason>,
    pub display_coverage: IfccadPolylineWidthDisplayCoverage,
}
#[derive(Clone, Copy)]
pub(crate) enum PolylineWidthSourceDirection {
    FromCad,
    ToCad,
}
type Source = IfccadGeometryEntitySource;
type Owner = IfccadGeometryOwner;
type Domain = IfccadGeometryDomain;
type Proofs = HashMap<(Domain, Source), BTreeMap<usize, IfccadDistanceInterval>>;
struct Collector<'a> {
    native: &'a IfccadDocument,
    cad: &'a CadDocument,
    maps: &'a IfccadMappings,
    direction: PolylineWidthSourceDirection,
    proofs: Proofs,
    bounds: IfccadBoundsAssessment,
    out: Vec<IfccadPolylineWidthAssessment>,
}
fn invalid(message: impl Into<String>) -> IfccadConversionError {
    IfccadConversionError::InvalidStructure(message.into())
}
fn local_cad(
    e: &EntityType,
) -> Result<(Vec<[f64; 3]>, IfccadPlanarPolylineWidths, bool), IfccadConversionError> {
    let widths = cad_geometry_convert::geometry::polyline_width::from_cad_polyline_widths(e)
        .map_err(|e| invalid(format!("retained width mapping: {e}")))?;
    match e {
        EntityType::LwPolyline(p) => Ok((
            p.vertices
                .iter()
                .map(|v| [v.location.x, v.location.y, v.bulge])
                .collect(),
            widths,
            p.is_closed,
        )),
        EntityType::Polyline2D(p) => Ok((
            p.vertices
                .iter()
                .map(|v| [v.location.x, v.location.y, v.bulge])
                .collect(),
            widths,
            p.flags.is_closed(),
        )),
        _ => Err(invalid("retained native polyline changed CAD kind")),
    }
}
impl Collector<'_> {
    fn identity(&self, owner: Owner, id: u64, actual: &EntityType) -> Source {
        match self.direction {
            PolylineWidthSourceDirection::FromCad => Source::CadEntity {
                handle: actual.common().handle,
                kind: if matches!(actual, EntityType::Dimension(_)) {
                    "DIMENSION".into()
                } else {
                    actual.as_entity().entity_type().into()
                },
            },
            PolylineWidthSourceDirection::ToCad => Source::NativeEntity {
                owner,
                entity_id: id,
            },
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn contents(
        &mut self,
        owner: Owner,
        entities: &[IfccadEntity],
        path: &mut Vec<Source>,
        ids: &mut Vec<u64>,
        root: IfccadScopeId,
        domain: Domain,
        part_path: Option<&str>,
    ) -> Result<(), IfccadConversionError> {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let Some(handle) = self.maps.entities.cad_handle(e.id) else {
                continue;
            };
            let actual = self
                .cad
                .get_entity(handle)
                .ok_or_else(|| invalid("missing retained width entity"))?;
            self.kind(
                owner,
                e.id,
                &e.kind,
                actual,
                self.identity(owner, e.id, actual),
                path,
                ids,
                root,
                domain,
                part_path,
            )?;
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn kind(
        &mut self,
        owner: Owner,
        id: u64,
        kind: &IfccadEntityKind,
        actual: &EntityType,
        identity: Source,
        path: &mut Vec<Source>,
        ids: &mut Vec<u64>,
        root: IfccadScopeId,
        domain: Domain,
        part_path: Option<&str>,
    ) -> Result<(), IfccadConversionError> {
        match kind {
            IfccadEntityKind::PlanarPolyline {
                widths,
                vertices,
                closed,
                ..
            } => {
                let effective = widths
                    .effective_widths(vertices.len())
                    .map_err(|e| invalid(e.to_string()))?;
                let source = if path.is_empty() {
                    identity
                } else {
                    Source::BlockOccurrence {
                        path: path.clone(),
                        leaf: Box::new(identity),
                    }
                };
                let count = if *closed {
                    vertices.len()
                } else {
                    vertices.len() - 1
                };
                let mut segments = Vec::new();
                for i in 0..count {
                    let wide = effective[i].iter().any(|v| *v != 0.);
                    let coincident = vertices[i] == vertices[(i + 1) % vertices.len()];
                    let (coverage, deviation, reason) = if !wide {
                        (IfccadPolylineWidthBandCoverage::ZeroWidth, None, None)
                    } else if coincident {
                        (
                            IfccadPolylineWidthBandCoverage::Unassessed,
                            None,
                            Some(Reason::CoincidentWideSegment),
                        )
                    } else {
                        let interval=self.proofs.get(&(domain,source.clone())).and_then(|p|p.get(&i)).copied().ok_or_else(||invalid(format!("missing committed width-band certificate for {source:?} segment {i}")))?;
                        (
                            IfccadPolylineWidthBandCoverage::Proven,
                            Some(interval),
                            None,
                        )
                    };
                    segments.push(IfccadPolylineWidthSegmentAssessment {
                        segment_index: i,
                        band_coverage: coverage,
                        deviation,
                        reason,
                    });
                }
                let native_view = kind
                    .as_shared_geometry()
                    .map_err(IfccadConversionError::CoreValidation)?
                    .ok_or_else(|| invalid("missing native planar geometry"))?;
                let mut reasons = geometry_kernel::polyline_width_display_bounds(native_view)
                    .map_err(|e| invalid(e.to_string()))?
                    .reasons;
                let (v, cw, cc) = local_cad(actual)?;
                if v.len() != vertices.len() || cc != *closed {
                    return Err(invalid(
                        "retained width mapping changed vertex count/closure",
                    ));
                }
                let frame = geometry_kernel::CoordinateFrame3::try_new(
                    geometry_kernel::Point3::new(0., 0., 0.),
                    geometry_kernel::Vector3::new(1., 0., 0.),
                    geometry_kernel::Vector3::new(0., 1., 0.),
                )
                .expect("identity frame");
                reasons.extend(
                    geometry_kernel::polyline_width_display_bounds(
                        geometry_kernel::GeometryRef::PlanarPolyline {
                            placement: frame,
                            vertices: geometry_kernel::PlanarVertices::Packed(&v),
                            widths: &cw,
                            closed: cc,
                        },
                    )
                    .map_err(|e| invalid(e.to_string()))?
                    .reasons,
                );
                let node = format!("/cad/d{}/e{id}/", self.native.drawing_id);
                reasons.extend(
                    self.bounds.scopes[&root]
                        .polyline_width_reasons
                        .iter()
                        .filter(|r| {
                            r.occurrence_path == *ids
                                && (r.source_path.starts_with(&node)
                                    || part_path.is_some_and(|p| r.source_path.starts_with(p)))
                        })
                        .map(|r| r.reason),
                );
                let mut unique = Vec::new();
                for reason in reasons {
                    if !unique.contains(&reason) {
                        unique.push(reason)
                    }
                }
                let surface = if unique.is_empty()
                    && segments
                        .iter()
                        .all(|s| s.band_coverage != IfccadPolylineWidthBandCoverage::Unassessed)
                {
                    IfccadPolylineWidthSurfaceCoverage::Proven
                } else {
                    IfccadPolylineWidthSurfaceCoverage::Unassessed
                };
                self.out.push(IfccadPolylineWidthAssessment {
                    source,
                    widths: widths.clone(),
                    effective_widths: effective,
                    segments,
                    surface_coverage: surface,
                    surface_reasons: unique,
                    display_coverage: IfccadPolylineWidthDisplayCoverage::Unassessed,
                });
            }
            IfccadEntityKind::BlockInstance { definition_id, .. } => {
                let drawing = self.native;
                let block = drawing
                    .blocks
                    .iter()
                    .find(|b| b.id == *definition_id)
                    .ok_or_else(|| invalid("missing retained width definition"))?;
                path.push(identity);
                ids.push(id);
                self.contents(
                    Owner::BlockDefinition(block.id),
                    &block.entities,
                    path,
                    ids,
                    root,
                    domain,
                    part_path,
                )?;
                ids.pop();
                path.pop();
            }
            IfccadEntityKind::Dimension(dimension) => {
                let Some(graphics) = &dimension.graphics else {
                    return Ok(());
                };
                let EntityType::Dimension(cad) = actual else {
                    return Ok(());
                };
                let Some(block) = self.cad.block_records.get(&cad.base().block_name) else {
                    return Ok(());
                };
                if block.entity_handles.len() != graphics.parts.len() {
                    return Err(invalid("width graphics picture part count changed"));
                }
                path.push(identity.clone());
                for (index, part) in graphics.parts.iter().enumerate() {
                    let actual = self
                        .cad
                        .get_entity(block.entity_handles[index])
                        .ok_or_else(|| invalid("missing width graphics part"))?;
                    let child_source = match self.direction {
                        PolylineWidthSourceDirection::FromCad => Source::CadEntity {
                            handle: actual.common().handle,
                            kind: actual.as_entity().entity_type().into(),
                        },
                        PolylineWidthSourceDirection::ToCad => identity.clone(),
                    };
                    let source = Source::DimensionGraphicsPart {
                        owner,
                        dimension: Box::new(identity.clone()),
                        part_index: index,
                        source: Box::new(child_source),
                    };
                    let node = format!("/cad/d{}/e{id}/graphics/p{index}", self.native.drawing_id);
                    self.kind(
                        owner,
                        id,
                        &part.kind.clone().into(),
                        actual,
                        source,
                        path,
                        ids,
                        root,
                        domain,
                        Some(&node),
                    )?;
                }
                path.pop();
            }
            _ => {}
        }
        Ok(())
    }
}
pub(crate) fn collect_polyline_widths(
    native: &IfccadDocument,
    cad: &CadDocument,
    maps: &IfccadMappings,
    direction: PolylineWidthSourceDirection,
    assessment: &IfccadGeometryAssessment,
) -> Result<Vec<IfccadPolylineWidthAssessment>, IfccadConversionError> {
    let mut proofs: Proofs = HashMap::new();
    for record in &assessment.verified_polyline_bands {
        let values = proofs
            .entry((record.domain, record.source.clone()))
            .or_default();
        for &(index, d) in &record.segments {
            values
                .entry(index)
                .and_modify(|old| {
                    old.lower = old.lower.min(d.lower);
                    old.upper = old.upper.max(d.upper);
                })
                .or_insert(d);
        }
    }
    let mut collector = Collector {
        native,
        cad,
        maps,
        direction,
        proofs,
        bounds: assess_ifccad_document_bounds(native)
            .map_err(IfccadConversionError::CoreValidation)?,
        out: vec![],
    };
    let owners = std::iter::once((
        Owner::ModelLayout(native.model.id),
        native.model.entities.as_slice(),
    ))
    .chain(
        native
            .paper_layouts
            .iter()
            .map(|p| (Owner::PaperLayout(p.id), p.entities.as_slice())),
    )
    .chain(
        native
            .blocks
            .iter()
            .map(|b| (Owner::BlockDefinition(b.id), b.entities.as_slice())),
    );
    for (owner, contents) in owners {
        let root = match owner {
            Owner::ModelLayout(id) | Owner::PaperLayout(id) => IfccadScopeId::Layout(id),
            Owner::BlockDefinition(id) => IfccadScopeId::BlockDefinition(id),
        };
        collector.contents(
            owner,
            contents,
            &mut vec![],
            &mut vec![],
            root,
            owner.domain(),
            None,
        )?;
    }
    Ok(collector.out)
}
