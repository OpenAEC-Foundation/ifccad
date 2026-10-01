//! Typed geometry preparation shared by the standalone conversion directions.
//!
//! Numerical preparation operates on OCDraw values, independent of encoding.

use crate::geometry::{
    self,
    blocks::{EvaluatedBlock, PairedCurve, PairedPoint},
    circular,
};
use crate::{
    ConversionEntitySource, ConversionGeometryAssessment, ConversionGeometryFailure,
    ConversionGeometryFailureReason as Reason, ConversionGeometryStage as Stage,
};
use cadcodec::{EntityType, Handle};
use ocdraw::ocdraw::{CoordinateFrame3, DrawingGeometry, Point3, Vector3};
use std::collections::BTreeMap;

struct Instance<K> {
    definition: K,
    source: EvaluatedBlock,
    target: EvaluatedBlock,
}
pub(super) struct ExchangeState<K> {
    pub assessment: ConversionGeometryAssessment,
    points: BTreeMap<K, Vec<PairedPoint>>,
    curves: BTreeMap<K, PairedCurve>,
    instances: BTreeMap<K, Instance<K>>,
    identities: BTreeMap<K, ConversionEntitySource>,
}
impl<K: Copy + Ord> ExchangeState<K> {
    pub fn new(assessment: ConversionGeometryAssessment) -> Self {
        Self {
            assessment,
            points: BTreeMap::new(),
            curves: BTreeMap::new(),
            instances: BTreeMap::new(),
            identities: BTreeMap::new(),
        }
    }
    pub fn affected_instances(
        &self,
        members: &BTreeMap<K, Vec<K>>,
        affected: &std::collections::BTreeSet<K>,
    ) -> BTreeMap<K, Vec<K>> {
        let mut result = BTreeMap::<K, Vec<K>>::new();
        for (&root, instance) in &self.instances {
            let mut pending = vec![instance.definition];
            let mut visited = std::collections::BTreeSet::new();
            while let Some(definition) = pending.pop() {
                if !visited.insert(definition) {
                    continue;
                }
                if affected.contains(&definition) {
                    result.entry(definition).or_default().push(root);
                }
                for child in members.get(&definition).into_iter().flatten() {
                    if let Some(instance) = self.instances.get(child) {
                        pending.push(instance.definition);
                    }
                }
            }
        }
        result
    }
    fn curve(
        &mut self,
        key: K,
        samples: Vec<(PairedPoint, num_rational::BigRational)>,
        curve: Option<PairedCurve>,
    ) -> Result<f64, Box<ConversionGeometryFailure>> {
        let identity = &self.identities[&key];
        let curve = curve.ok_or_else(|| {
            self.assessment.failure(
                identity,
                None,
                Stage::DeviationAssessment,
                Reason::DeviationBoundOutOfRange,
            )
        })?;
        let mut maximum = 0.0_f64;
        let mut points = Vec::new();
        for (index, (point, d2)) in samples.into_iter().enumerate() {
            maximum = maximum.max(self.assessment.check(identity, index, &d2)?);
            points.push(point);
        }
        maximum = maximum.max(self.assessment.check_curve(identity, &curve)?);
        self.assessment
            .record(identity.clone(), points.len(), maximum);
        self.points.insert(key, points);
        self.curves.insert(key, curve);
        Ok(maximum)
    }
    fn exact(&mut self, key: K, points: impl IntoIterator<Item = [f64; 3]>) {
        let points = points
            .into_iter()
            .map(PairedPoint::exact)
            .collect::<Vec<_>>();
        self.assessment
            .record(self.identities[&key].clone(), points.len(), 0.0);
        self.points.insert(key, points);
    }
    // The source and validated drawing are acyclic. Include unused definitions,
    // and apply nested transforms in reverse path order to retain correlation.
    pub fn assess_occurrences(
        &mut self,
        members: &BTreeMap<K, Vec<K>>,
    ) -> Result<Vec<(K, f64)>, Box<ConversionGeometryFailure>> {
        let mut rounded = Vec::new();
        for (&root, instance) in &self.instances {
            let mut stack = vec![(instance.definition, vec![root])];
            while let Some((definition, path)) = stack.pop() {
                for &leaf in members.get(&definition).into_iter().flatten() {
                    if let Some(nested) = self.instances.get(&leaf) {
                        let mut path = path.clone();
                        path.push(leaf);
                        stack.push((nested.definition, path));
                    } else if let Some(points) = self.points.get(&leaf) {
                        let identity = ConversionEntitySource::BlockOccurrence {
                            path: path
                                .iter()
                                .map(|key| self.identities[key].clone())
                                .collect(),
                            leaf: Box::new(self.identities[&leaf].clone()),
                        };
                        let mut maximum = 0.0_f64;
                        for (index, point) in points.iter().enumerate() {
                            let mut point = point.clone();
                            for key in path.iter().rev() {
                                let pair = &self.instances[key];
                                point.apply(&pair.source, &pair.target);
                            }
                            let (lower, upper) = point.squared_deviation();
                            maximum = maximum.max(
                                self.assessment
                                    .check_interval(&identity, index, &lower, &upper)?,
                            );
                        }
                        if let Some(curve) = self.curves.get(&leaf) {
                            let mut curve = curve.clone();
                            for key in path.iter().rev() {
                                let pair = &self.instances[key];
                                curve.apply(&pair.source, &pair.target);
                            }
                            maximum = maximum.max(self.assessment.check_curve(&identity, &curve)?);
                        }
                        self.assessment.record(identity, points.len(), maximum);
                        if maximum > 0.0 {
                            rounded.push((root, maximum));
                        }
                    }
                }
            }
        }
        Ok(rounded)
    }
}
fn xyz(p: cadcodec::Vector3) -> [f64; 3] {
    [p.x, p.y, p.z]
}

pub(super) fn from_cad(
    entity: &EntityType,
    definitions: &BTreeMap<Handle, u32>,
    document: &cadcodec::CadDocument,
    state: &mut ExchangeState<Handle>,
) -> Result<(DrawingGeometry, f64, bool), Box<ConversionGeometryFailure>> {
    let key = entity.common().handle;
    let identity = ConversionEntitySource::CadEntity {
        handle: key,
        kind: entity.as_entity().entity_type().into(),
    };
    state.identities.insert(key, identity.clone());
    let range_failure = || {
        state.assessment.failure(
            &identity,
            None,
            Stage::TargetConstruction,
            Reason::TargetCoordinateOutOfRange,
        )
    };
    let mut maximum = 0.0;
    let mut normalized = false;
    let result = match entity {
        EntityType::Line(line) => {
            state.exact(key, [xyz(line.start), xyz(line.end)]);
            DrawingGeometry::Line {
                start: xyz(line.start),
                end: xyz(line.end),
            }
        }
        EntityType::Point(point) => {
            let basis = geometry::cad_plane(point.normal).expect("classified point plane");
            let (s, c) = point.x_axis_angle.sin_cos();
            let rotated = |a: f64, b: f64| {
                cadcodec::Vector3::new(
                    a * basis.u[0] + b * basis.v[0],
                    a * basis.u[1] + b * basis.v[1],
                    a * basis.u[2] + b * basis.v[2],
                )
            };
            let (x, y) = geometry::orthonormal_pair(rotated(c, s), rotated(-s, c))
                .expect("classified point axes");
            let placement = CoordinateFrame3::try_new(
                Point3::new(point.location.x, point.location.y, point.location.z),
                Vector3::new(x.x, x.y, x.z),
                Vector3::new(y.x, y.y, y.z),
            )
            .expect("classified point frame");
            normalized = geometry::stored_normal(placement) != Some(point.normal);
            state.exact(key, [xyz(point.location)]);
            DrawingGeometry::Point { placement }
        }
        EntityType::Circle(circle) => {
            let placement = circular::from_cad_ocs(circle.center, circle.normal)
                .expect("classified circle frame");
            let samples = circular::export_circle_sample_pairs(circle, placement)
                .ok_or_else(range_failure)?;
            maximum = state.curve(
                key,
                samples,
                circular::export_circle_curve(circle, placement),
            )?;
            normalized = geometry::stored_normal(placement) != Some(circle.normal);
            DrawingGeometry::Circle {
                placement,
                radius: circle.radius,
            }
        }
        EntityType::Arc(arc) => {
            let placement =
                circular::from_cad_ocs(arc.center, arc.normal).expect("classified arc frame");
            let sweep = (arc.end_angle - arc.start_angle).rem_euclid(std::f64::consts::TAU);
            let samples = circular::export_arc_sample_pairs(arc, placement, sweep)
                .ok_or_else(range_failure)?;
            maximum = state.curve(
                key,
                samples,
                circular::export_arc_curve(arc, placement, sweep),
            )?;
            normalized = geometry::stored_normal(placement) != Some(arc.normal);
            DrawingGeometry::Arc {
                placement,
                radius: arc.radius,
                start_parameter: arc.start_angle,
                sweep_parameter: sweep,
            }
        }
        EntityType::Ellipse(ellipse) => {
            let (placement, major, minor) =
                circular::from_cad_ellipse(ellipse).expect("classified ellipse frame");
            let difference = ellipse.end_parameter - ellipse.start_parameter;
            let full = difference == std::f64::consts::TAU;
            let sweep = if full {
                difference
            } else {
                difference.rem_euclid(std::f64::consts::TAU)
            };
            let samples = circular::export_ellipse_sample_pairs(
                ellipse,
                placement,
                major,
                minor,
                ellipse.start_parameter,
                sweep,
            )
            .ok_or_else(range_failure)?;
            maximum = state.curve(
                key,
                samples,
                circular::export_ellipse_curve(ellipse, placement, major, minor, sweep),
            )?;
            normalized = geometry::stored_normal(placement) != Some(ellipse.normal);
            DrawingGeometry::Ellipse {
                placement,
                semi_major_radius: major,
                semi_minor_radius: minor,
                arc: (!full).then_some((ellipse.start_parameter, sweep)),
            }
        }
        EntityType::LwPolyline(poly) => {
            planar_from_cad(poly, key, state, &mut maximum, &mut normalized)?
        }
        EntityType::Polyline2D(poly) => {
            let mut prepared = cadcodec::LwPolyline::from_points(
                poly.vertices
                    .iter()
                    .map(|v| cadcodec::Vector2::new(v.location.x, v.location.y))
                    .collect(),
            );
            prepared.common = poly.common.clone();
            prepared.normal = poly.normal;
            prepared.elevation = poly.elevation;
            prepared.is_closed = poly.flags.is_closed();
            for (a, b) in prepared.vertices.iter_mut().zip(&poly.vertices) {
                a.bulge = b.bulge;
            }
            planar_from_cad(&prepared, key, state, &mut maximum, &mut normalized)?
        }
        EntityType::Polyline3D(poly) => {
            let vertices = poly
                .vertices
                .iter()
                .map(|v| xyz(v.position))
                .collect::<Vec<_>>();
            state.exact(key, vertices.iter().copied());
            DrawingGeometry::SpatialPolyline {
                vertices,
                closed: poly.flags.closed,
            }
        }
        EntityType::Polyline(poly) => {
            let vertices = poly
                .vertices
                .iter()
                .map(|v| xyz(v.location))
                .collect::<Vec<_>>();
            state.exact(key, vertices.iter().copied());
            DrawingGeometry::SpatialPolyline {
                vertices,
                closed: poly.flags.is_closed(),
            }
        }
        EntityType::Insert(insert) => {
            let record = document
                .block_records
                .get(&insert.block_name)
                .expect("checked block reference");
            let (native, source, target) =
                geometry::blocks::from_cad_instance(insert, record.base_point, &state.assessment)?;
            normalized = geometry::stored_normal(native.placement()) != Some(insert.normal);
            state.instances.insert(
                key,
                Instance {
                    definition: record.handle,
                    source,
                    target,
                },
            );
            DrawingGeometry::BlockInstance {
                definition_scope_id: definitions[&record.handle],
                transform: native,
            }
        }
        _ => unreachable!("classified geometry family"),
    };
    Ok((result, maximum, normalized))
}
fn planar_from_cad(
    poly: &cadcodec::LwPolyline,
    key: Handle,
    state: &mut ExchangeState<Handle>,
    maximum: &mut f64,
    normalized: &mut bool,
) -> Result<DrawingGeometry, Box<ConversionGeometryFailure>> {
    let (placement, bound, changed) = geometry::from_cad(poly, &mut state.assessment)?;
    *maximum = bound;
    *normalized = changed;
    state
        .points
        .insert(key, geometry::blocks::polyline_pairs(poly, placement));
    Ok(DrawingGeometry::PlanarPolyline {
        placement,
        vertices: poly
            .vertices
            .iter()
            .map(|v| [v.location.x, v.location.y, v.bulge])
            .collect(),
        closed: poly.is_closed,
    })
}

pub(super) fn to_cad(
    source: &ocdraw::ocdraw::DrawingGeometricEntity,
    owner_scope_id: u32,
    blocks: &BTreeMap<u64, ocdraw::ocdraw::DrawingBlockDefinition>,
    state: &mut ExchangeState<u64>,
) -> Result<(EntityType, f64, bool), super::import::DirectImportError> {
    let key = source.id();
    let identity = ConversionEntitySource::DrawingEntity {
        scope_id: owner_scope_id,
        entity_id: key,
    };
    state.identities.insert(key, identity.clone());
    let fail = || {
        state.assessment.failure(
            &identity,
            None,
            Stage::TargetConstruction,
            Reason::CadAxisEvaluationFailed,
        )
    };
    let range_failure = || {
        state.assessment.failure(
            &identity,
            None,
            Stage::TargetConstruction,
            Reason::TargetCoordinateOutOfRange,
        )
    };
    let mut maximum = 0.0;
    let mut changed = false;
    let entity = match source.geometry() {
        DrawingGeometry::Line { start, end } => {
            state.exact(key, [*start, *end]);
            EntityType::Line(cadcodec::Line::from_coords(
                start[0], start[1], start[2], end[0], end[1], end[2],
            ))
        }
        DrawingGeometry::Point { placement } => {
            let native = *placement;
            let normal = geometry::stored_normal(native).ok_or_else(fail)?;
            let basis = geometry::cad_plane(normal).ok_or_else(fail)?;
            let x = placement.x_axis();
            let o = placement.origin();
            let dot = |a: [f64; 3]| x.x() * a[0] + x.y() * a[1] + x.z() * a[2];
            let mut target = cadcodec::Point::from_coords(o.x(), o.y(), o.z());
            target.normal = normal;
            target.x_axis_angle = dot(basis.v).atan2(dot(basis.u));
            state.exact(key, [[o.x(), o.y(), o.z()]]);
            EntityType::Point(target)
        }
        DrawingGeometry::Circle { placement, radius } => {
            let native = *placement;
            let (center, normal, phase) = circular::to_cad_ocs(native, false).ok_or_else(fail)?;
            let mut target = cadcodec::Circle::from_center_radius(center, *radius);
            target.normal = normal;
            let samples = circular::circle_sample_pairs(native, *radius, phase, &target)?;
            maximum = state.curve(
                key,
                samples,
                circular::import_circle_curve(native, *radius, phase, &target),
            )?;
            EntityType::Circle(target)
        }
        DrawingGeometry::Arc {
            placement,
            radius,
            start_parameter,
            sweep_parameter,
        } => {
            let native = *placement;
            let sweep = *sweep_parameter;
            let (center, normal, phase) =
                circular::to_cad_ocs(native, sweep < 0.0).ok_or_else(fail)?;
            let start = phase
                + if sweep < 0.0 {
                    -start_parameter
                } else {
                    *start_parameter
                };
            let mut target = cadcodec::Arc::from_center_radius_angles(
                center,
                *radius,
                start,
                start + sweep.abs(),
            );
            target.normal = normal;
            let samples =
                circular::arc_sample_pairs(native, *radius, *start_parameter, sweep, &target)?;
            maximum = state.curve(
                key,
                samples,
                circular::import_arc_curve(native, *radius, *start_parameter, sweep, &target),
            )?;
            EntityType::Arc(target)
        }
        DrawingGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => {
            let native = *placement;
            let (start, sweep) = arc.unwrap_or((0.0, std::f64::consts::TAU));
            let mut target = circular::to_cad_ellipse(
                native,
                *semi_major_radius,
                *semi_minor_radius,
                sweep < 0.0,
            )
            .ok_or_else(fail)?;
            target.start_parameter = if sweep < 0.0 { -start } else { start };
            target.end_parameter = target.start_parameter + sweep.abs();
            let samples = circular::ellipse_sample_pairs(
                native,
                *semi_major_radius,
                *semi_minor_radius,
                start,
                sweep,
                &target,
            )
            .ok_or_else(range_failure)?;
            maximum = state.curve(
                key,
                samples,
                circular::import_ellipse_curve(
                    native,
                    *semi_major_radius,
                    *semi_minor_radius,
                    start,
                    sweep,
                    &target,
                ),
            )?;
            EntityType::Ellipse(target)
        }
        DrawingGeometry::PlanarPolyline {
            placement,
            vertices,
            closed,
        } => {
            let native = *placement;
            let (target, bound, parameterization) =
                geometry::to_cad_parts(native, vertices, *closed, identity, &mut state.assessment)?;
            maximum = bound;
            changed = parameterization;
            state.points.insert(
                key,
                geometry::blocks::import_polyline_parts(native, vertices, *closed, &target),
            );
            EntityType::LwPolyline(target)
        }
        DrawingGeometry::SpatialPolyline { vertices, closed } => {
            let mut target = cadcodec::entities::Polyline3D::from_points(
                vertices
                    .iter()
                    .map(|v| cadcodec::Vector3::new(v[0], v[1], v[2]))
                    .collect(),
            );
            target.flags.closed = *closed;
            state.exact(key, vertices.iter().copied());
            EntityType::Polyline3D(target)
        }
        DrawingGeometry::BlockInstance {
            definition_scope_id,
            transform,
        } => {
            let definition = &blocks[&u64::from(*definition_scope_id)];
            let (target, source_map, target_map, parameterization) =
                geometry::blocks::to_cad_instance_parts(
                    *transform,
                    &definition.name,
                    definition.base_point,
                    identity,
                    &state.assessment,
                )?;
            changed = parameterization;
            state.instances.insert(
                key,
                Instance {
                    definition: u64::from(*definition_scope_id),
                    source: source_map,
                    target: target_map,
                },
            );
            EntityType::Insert(target)
        }
    };
    Ok((entity, maximum, changed))
}
