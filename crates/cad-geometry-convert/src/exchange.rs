//! Shared identity-safe registration and occurrence-space accuracy propagation.
use crate::geometry::blocks::{EvaluatedBlock, PairedCurve, PairedPoint};
use crate::{GeometryAssessment, GeometryFailure};
use crate::{GeometryFailureReason as Reason, GeometryStage as Stage};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct OccurrenceAssessment<K, S> {
    pub root: K,
    pub path: Vec<K>,
    pub leaf: S,
    pub deviation: crate::DistanceInterval,
}
#[derive(Clone)]
struct Instance<K> {
    definition: K,
    source: EvaluatedBlock,
    target: EvaluatedBlock,
}
pub struct ExchangeState<K, S> {
    pub assessment: GeometryAssessment<S>,
    points: BTreeMap<K, Vec<PairedPoint>>,
    curves: BTreeMap<K, Vec<PairedCurve>>,
    instances: BTreeMap<K, Instance<K>>,
    identities: BTreeMap<K, S>,
}
impl<K: Copy + Ord, S: crate::GeometrySource> ExchangeState<K, S> {
    pub fn new(assessment: GeometryAssessment<S>) -> Self {
        Self {
            assessment,
            points: BTreeMap::new(),
            curves: BTreeMap::new(),
            instances: BTreeMap::new(),
            identities: BTreeMap::new(),
        }
    }

    pub fn register_identity(&mut self, key: K, source: S) {
        self.identities.insert(key, source);
    }
    pub fn identity(&self, key: K) -> &S {
        &self.identities[&key]
    }
    pub fn register_points(&mut self, key: K, points: Vec<PairedPoint>) {
        self.points.insert(key, points);
    }
    pub fn record_instance_parts(
        &mut self,
        key: K,
        definition: K,
        source: EvaluatedBlock,
        target: EvaluatedBlock,
    ) {
        self.instances.insert(
            key,
            Instance {
                definition,
                source,
                target,
            },
        );
    }
    pub fn record_geometry(
        &mut self,
        key: K,
        source: S,
        pair: crate::GeometryPair,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
        let mut maximum = 0.0_f64;
        for (index, (_, d2)) in pair.points.iter().enumerate() {
            maximum = maximum.max(self.assessment.check(&source, index, d2)?);
        }
        for curve in &pair.curves {
            maximum = maximum.max(self.assessment.check_curve(&source, curve)?);
        }
        self.assessment
            .record(source.clone(), pair.points.len(), maximum);
        self.identities.insert(key, source);
        self.points
            .insert(key, pair.points.into_iter().map(|(p, _)| p).collect());
        self.curves.insert(key, pair.curves);
        Ok(maximum)
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
    pub fn curve(
        &mut self,
        key: K,
        samples: Vec<(PairedPoint, num_rational::BigRational)>,
        curve: Option<PairedCurve>,
    ) -> Result<f64, Box<GeometryFailure<S>>> {
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
        self.curves.insert(key, vec![curve]);
        Ok(maximum)
    }
    pub fn exact(&mut self, key: K, points: impl IntoIterator<Item = [f64; 3]>) {
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
    ) -> Result<Vec<(K, f64)>, Box<GeometryFailure<S>>> {
        let roots = self.instances.keys().copied().collect::<Vec<_>>();
        let mut assessment = self.assessment.clone();
        let records = self.assess_roots(members, &roots, &mut assessment)?;
        self.assessment = assessment;
        Ok(records
            .into_iter()
            .filter(|r| r.deviation.upper() > 0.)
            .map(|r| (r.root, r.deviation.upper()))
            .collect())
    }
    /// Input ownership/reference graphs must already be validated as acyclic.
    pub fn assess_roots(
        &self,
        members: &BTreeMap<K, Vec<K>>,
        roots: &[K],
        assessment: &mut GeometryAssessment<S>,
    ) -> Result<Vec<OccurrenceAssessment<K, S>>, Box<GeometryFailure<S>>> {
        let mut records = Vec::new();
        for &root in roots {
            let Some(instance) = self.instances.get(&root) else {
                continue;
            };
            let mut stack = vec![(instance.definition, vec![root])];
            while let Some((definition, path)) = stack.pop() {
                for &leaf in members.get(&definition).into_iter().flatten() {
                    if let Some(nested) = self.instances.get(&leaf) {
                        let mut path = path.clone();
                        path.push(leaf);
                        stack.push((nested.definition, path));
                    } else if let Some(points) = self.points.get(&leaf) {
                        let identity = S::occurrence(
                            path.iter().map(|k| self.identities[k].clone()).collect(),
                            self.identities[&leaf].clone(),
                        );
                        let mut maximum = 0.0_f64;
                        let mut lower = 0.0_f64;
                        for (index, p) in points.iter().enumerate() {
                            let mut p = p.clone();
                            for key in path.iter().rev() {
                                let pair = &self.instances[key];
                                p.apply(&pair.source, &pair.target);
                            }
                            let (lo, hi) = p.squared_deviation();
                            maximum =
                                maximum.max(assessment.check_interval(&identity, index, &lo, &hi)?);
                            if let Some((lo, _)) = crate::geometry::numeric::sqrt_interval(&lo) {
                                lower = lower.max(lo);
                            }
                        }
                        if let Some(curves) = self.curves.get(&leaf) {
                            for curve in curves {
                                let mut curve = curve.clone();
                                for key in path.iter().rev() {
                                    let pair = &self.instances[key];
                                    curve.apply(&pair.source, &pair.target);
                                }
                                maximum = maximum.max(assessment.check_curve(&identity, &curve)?);
                            }
                        }
                        assessment.record(identity, points.len(), maximum);
                        records.push(OccurrenceAssessment {
                            root,
                            path: path.clone(),
                            leaf: self.identities[&leaf].clone(),
                            deviation: crate::DistanceInterval {
                                lower,
                                upper: maximum,
                            },
                        });
                    }
                }
            }
        }
        Ok(records)
    }
}
