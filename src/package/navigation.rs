use super::analysis::ValidatedIfccadPackage;
use super::appearance::{
    appearance_mode, AppearanceColorRef, AppearanceMode, AppearanceProperty, LinePatternRef,
};
use crate::ifcdr::{AppearanceId, IfcdrResourceRef, LayerId, ScopeRef, ValidatedIfcdrResource};
use serde_json::Value;

impl ValidatedIfccadPackage {
    pub fn drawing_sets(&self) -> impl Iterator<Item = DrawingSetRef<'_>> {
        typed_nodes(self, "openaec:DrawingSet").map(|node_index| DrawingSetRef {
            package: self,
            node_index,
        })
    }

    pub fn drawings(&self) -> impl Iterator<Item = DrawingRef<'_>> {
        typed_nodes(self, "openaec:Drawing").map(|node_index| DrawingRef {
            package: self,
            node_index,
        })
    }

    #[cfg(test)]
    pub(crate) fn layouts(&self) -> impl Iterator<Item = DrawingLayoutRef<'_>> {
        typed_nodes(self, "openaec:DrawingLayout").map(|node_index| DrawingLayoutRef {
            package: self,
            node_index,
        })
    }

    #[cfg(test)]
    pub(crate) fn geometry_representations(
        &self,
    ) -> impl Iterator<Item = GeometryRepresentationRef<'_>> {
        typed_nodes(self, "openaec:DrawingGeometryRepresentation").map(|node_index| {
            GeometryRepresentationRef {
                package: self,
                node_index,
            }
        })
    }

    pub(crate) fn geometry_representation(
        &self,
        path: &str,
    ) -> Option<GeometryRepresentationRef<'_>> {
        self.typed_node(path, "openaec:DrawingGeometryRepresentation")
            .map(|node_index| GeometryRepresentationRef {
                package: self,
                node_index,
            })
    }

    pub(crate) fn ifcdr_resource(&self, uri: &str) -> Option<&ValidatedIfcdrResource> {
        self.evidence()
            .validated_ifcdr_resources
            .get(uri)
            .map(AsRef::as_ref)
    }

    #[cfg(test)]
    pub(crate) fn ifcx_node(&self, path: &str) -> Option<&Value> {
        self.evidence()
            .node_indices_by_path
            .get(path)
            .and_then(|index| nodes(self).get(*index))
    }

    fn typed_node(&self, path: &str, expected_type: &str) -> Option<usize> {
        let index = *self.evidence().node_indices_by_path.get(path)?;
        (nodes(self).get(index)?.get("type").and_then(Value::as_str) == Some(expected_type))
            .then_some(index)
    }

    fn drawing(&self, path: &str) -> Option<DrawingRef<'_>> {
        self.typed_node(path, "openaec:Drawing")
            .map(|node_index| DrawingRef {
                package: self,
                node_index,
            })
    }

    fn layout(&self, path: &str) -> Option<DrawingLayoutRef<'_>> {
        self.typed_node(path, "openaec:DrawingLayout")
            .map(|node_index| DrawingLayoutRef {
                package: self,
                node_index,
            })
    }

    fn layer(&self, path: &str, id: LayerId) -> Option<LayerRef<'_>> {
        self.typed_node(path, "openaec:Layer")
            .map(|node_index| LayerRef {
                package: self,
                node_index,
                id,
            })
    }

    fn appearance(&self, path: &str) -> Option<AppearanceRef<'_>> {
        self.typed_node(path, "openaec:Appearance")
            .map(|node_index| AppearanceRef {
                package: self,
                node_index,
            })
    }
}

fn nodes(package: &ValidatedIfccadPackage) -> &[Value] {
    package
        .loaded()
        .package()
        .entrypoint
        .value()
        .get("data")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .expect("validated IFCX data array")
}

fn typed_nodes<'a>(
    package: &'a ValidatedIfccadPackage,
    expected_type: &'static str,
) -> impl Iterator<Item = usize> + 'a {
    nodes(package)
        .iter()
        .enumerate()
        .filter_map(move |(index, node)| {
            (node.get("type").and_then(Value::as_str) == Some(expected_type)).then_some(index)
        })
}

macro_rules! node_ref {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name<'a> {
            package: &'a ValidatedIfccadPackage,
            node_index: usize,
        }

        impl<'a> $name<'a> {
            pub(crate) fn node(&self) -> &'a Value {
                &nodes(self.package)[self.node_index]
            }

            pub fn path(&self) -> &'a str {
                self.node()["path"].as_str().expect("validated node path")
            }
        }
    };
}

node_ref!(DrawingSetRef);
node_ref!(DrawingRef);
node_ref!(DrawingLayoutRef);
node_ref!(GeometryRepresentationRef);
node_ref!(AppearanceRef);

#[derive(Clone, Copy)]
pub struct LayerRef<'a> {
    package: &'a ValidatedIfccadPackage,
    node_index: usize,
    id: LayerId,
}

impl<'a> LayerRef<'a> {
    fn node(&self) -> &'a Value {
        &nodes(self.package)[self.node_index]
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    pub fn path(&self) -> &'a str {
        self.node()["path"].as_str().expect("validated node path")
    }
}

impl<'a> DrawingSetRef<'a> {
    pub fn drawings(&self) -> impl Iterator<Item = DrawingRef<'a>> + 'a {
        let package = self.package;
        self.node()
            .pointer("/children/Drawings")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .filter_map(move |path| package.drawing(path.as_str()?))
    }
}

impl<'a> DrawingRef<'a> {
    pub fn representation(&self) -> GeometryRepresentationRef<'a> {
        self.node()
            .pointer("/children/Representation")
            .and_then(Value::as_str)
            .and_then(|path| self.package.geometry_representation(path))
            .expect("validated drawing representation")
    }

    pub fn layouts(&self) -> impl Iterator<Item = DrawingLayoutRef<'a>> + 'a {
        let package = self.package;
        self.node()
            .pointer("/children/Layouts")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .filter_map(move |path| package.layout(path.as_str()?))
    }
}

impl<'a> DrawingLayoutRef<'a> {
    pub fn name(&self) -> &'a str {
        self.node()
            .pointer("/attributes/name")
            .and_then(Value::as_str)
            .expect("validated layout name")
    }

    pub fn kind(&self) -> DrawingLayoutKind {
        match self
            .node()
            .pointer("/attributes/kind")
            .and_then(Value::as_str)
            .expect("validated layout kind")
        {
            "model" => DrawingLayoutKind::Model,
            "paper" => DrawingLayoutKind::Paper,
            _ => unreachable!("validated drawing layout kind"),
        }
    }

    pub fn representation(&self) -> GeometryRepresentationRef<'a> {
        let binding = self
            .package
            .evidence()
            .bindings
            .layout_by_path
            .get(self.path())
            .expect("validated layout binding");
        self.package
            .geometry_representation(&binding.representation_path)
            .expect("validated layout representation")
    }

    pub fn scope(&self) -> ScopeRef<'a> {
        let binding = self
            .package
            .evidence()
            .bindings
            .layout_by_path
            .get(self.path())
            .expect("validated layout binding");
        self.package
            .ifcdr_resource(&binding.ifcdr_uri)
            .expect("validated layout IFCDR resource")
            .scope(binding.scope_id)
            .expect("validated layout scope")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingLayoutKind {
    Model,
    Paper,
}

impl<'a> GeometryRepresentationRef<'a> {
    pub fn role(&self) -> &'a str {
        self.node()
            .pointer("/attributes/geometry/role")
            .and_then(Value::as_str)
            .expect("validated representation role")
    }

    pub fn uri(&self) -> &'a str {
        self.node()
            .pointer("/attributes/geometry/uri")
            .and_then(Value::as_str)
            .expect("validated representation URI")
    }

    fn validated_resource(&self) -> &'a ValidatedIfcdrResource {
        self.package.evidence().bindings.geometry_ifcdr_by_path[self.path()].as_ref()
    }

    pub fn resource(&self) -> IfcdrResourceRef<'a> {
        IfcdrResourceRef::new(self.validated_resource())
    }

    pub fn layers(&self) -> impl ExactSizeIterator<Item = LayerRef<'a>> + 'a {
        let package = self.package;
        let uri = self.uri().to_owned();
        self.validated_resource()
            .bindings()
            .layers()
            .map(move |binding| {
                let id = binding.id();
                let path = package
                    .evidence()
                    .bindings
                    .ifcx_layer_by_ifcdr_id
                    .get(&(uri.clone(), id))
                    .expect("validated IFCX layer binding");
                package.layer(path, id).expect("validated IFCX layer")
            })
    }

    pub fn layer(&self, id: LayerId) -> Option<LayerRef<'a>> {
        let path = self
            .package
            .evidence()
            .bindings
            .ifcx_layer_by_ifcdr_id
            .get(&(self.uri().to_owned(), id))?;
        self.package.layer(path, id)
    }

    pub fn appearance(&self, id: AppearanceId) -> Option<AppliedAppearanceRef<'a>> {
        self.validated_resource().appearance_binding(id)?;
        Some(AppliedAppearanceRef {
            package: self.package,
            resource: self.validated_resource(),
            id,
        })
    }
}

impl<'a> LayerRef<'a> {
    pub fn name(&self) -> &'a str {
        self.node()
            .pointer("/attributes/name")
            .and_then(Value::as_str)
            .expect("validated layer name")
    }

    pub fn visible(&self) -> bool {
        self.node()
            .pointer("/attributes/visible")
            .and_then(Value::as_bool)
            .expect("validated layer visibility")
    }

    pub fn appearance(&self) -> Option<AppearanceRef<'a>> {
        self.node()
            .pointer("/attributes/appearance")
            .and_then(Value::as_str)
            .and_then(|path| self.package.appearance(path))
    }
}

impl<'a> AppearanceRef<'a> {
    pub fn name(&self) -> &'a str {
        self.node()
            .pointer("/attributes/name")
            .and_then(Value::as_str)
            .expect("validated appearance name")
    }

    pub fn color(&self) -> AppearanceColorRef<'a> {
        AppearanceColorRef::new(
            self.node()
                .pointer("/attributes/color/value")
                .expect("validated appearance color"),
        )
    }

    pub fn opacity(&self) -> f64 {
        self.node()
            .pointer("/attributes/opacity/value")
            .and_then(Value::as_f64)
            .expect("validated appearance opacity")
    }

    pub fn line_pattern(&self) -> LinePatternRef<'a> {
        LinePatternRef::Name(
            self.node()
                .pointer("/attributes/linePattern/value")
                .and_then(Value::as_str)
                .expect("validated appearance line pattern"),
        )
    }

    pub fn line_weight(&self) -> f64 {
        self.node()
            .pointer("/attributes/lineWeight/value")
            .and_then(Value::as_f64)
            .expect("validated appearance line weight")
    }
}

/// Resource-local appearance binding with independently inherited properties.
#[derive(Clone, Copy)]
pub struct AppliedAppearanceRef<'a> {
    package: &'a ValidatedIfccadPackage,
    resource: &'a ValidatedIfcdrResource,
    id: AppearanceId,
}

impl<'a> AppliedAppearanceRef<'a> {
    pub fn id(&self) -> AppearanceId {
        self.id
    }

    pub fn ifcx_definition(&self) -> Option<AppearanceRef<'a>> {
        self.resource
            .appearance_binding(self.id)
            .expect("validated appearance binding")
            .ifcx_appearance()
            .and_then(|path| self.package.appearance(path))
    }

    pub fn color(&self) -> AppearanceProperty<AppearanceColorRef<'_>> {
        let binding = self
            .resource
            .appearance_binding(self.id)
            .expect("validated appearance binding");
        match appearance_mode(binding.color_mode()).expect("validated color mode") {
            AppearanceMode::ByLayer => AppearanceProperty::ByLayer,
            AppearanceMode::ByBlock => AppearanceProperty::ByBlock,
            AppearanceMode::Explicit => {
                let override_value = binding
                    .override_id()
                    .and_then(|id| self.resource.appearance_override(id))
                    .and_then(|value| value.color());
                let value = override_value.or_else(|| {
                    self.ifcx_definition()
                        .map(|appearance| appearance.node())
                        .and_then(|node| node.pointer("/attributes/color/value"))
                });
                AppearanceProperty::Explicit(AppearanceColorRef::new(
                    value.expect("validated explicit appearance color"),
                ))
            }
        }
    }

    pub fn opacity(&self) -> AppearanceProperty<f64> {
        let binding = self
            .resource
            .appearance_binding(self.id)
            .expect("validated appearance binding");
        match appearance_mode(binding.opacity_mode()).expect("validated opacity mode") {
            AppearanceMode::ByLayer => AppearanceProperty::ByLayer,
            AppearanceMode::ByBlock => AppearanceProperty::ByBlock,
            AppearanceMode::Explicit => {
                let override_value = binding
                    .override_id()
                    .and_then(|id| self.resource.appearance_override(id))
                    .and_then(|value| value.opacity());
                let value = override_value.or_else(|| {
                    self.ifcx_definition()
                        .map(|appearance| appearance.opacity())
                });
                AppearanceProperty::Explicit(value.expect("validated explicit appearance opacity"))
            }
        }
    }

    pub fn line_pattern(&self) -> AppearanceProperty<LinePatternRef<'_>> {
        let binding = self
            .resource
            .appearance_binding(self.id)
            .expect("validated appearance binding");
        match appearance_mode(binding.line_pattern_mode()).expect("validated line-pattern mode") {
            AppearanceMode::ByLayer => AppearanceProperty::ByLayer,
            AppearanceMode::ByBlock => AppearanceProperty::ByBlock,
            AppearanceMode::Explicit => {
                let override_value = binding
                    .override_id()
                    .and_then(|id| self.resource.appearance_override(id))
                    .and_then(|value| value.ifcx_line_pattern())
                    .map(LinePatternRef::IfcxIdentity);
                let value = override_value.or_else(|| {
                    self.ifcx_definition()
                        .map(|appearance| appearance.line_pattern())
                });
                AppearanceProperty::Explicit(
                    value.expect("validated explicit appearance line pattern"),
                )
            }
        }
    }

    pub fn line_weight(&self) -> AppearanceProperty<f64> {
        let binding = self
            .resource
            .appearance_binding(self.id)
            .expect("validated appearance binding");
        match appearance_mode(binding.line_weight_mode()).expect("validated line-weight mode") {
            AppearanceMode::ByLayer => AppearanceProperty::ByLayer,
            AppearanceMode::ByBlock => AppearanceProperty::ByBlock,
            AppearanceMode::Explicit => {
                let override_value = binding
                    .override_id()
                    .and_then(|id| self.resource.appearance_override(id))
                    .and_then(|value| value.line_weight());
                let value = override_value.or_else(|| {
                    self.ifcx_definition()
                        .map(|appearance| appearance.line_weight())
                });
                AppearanceProperty::Explicit(
                    value.expect("validated explicit appearance line weight"),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::package::validation::load_directory_package;
    use crate::package::DIRECTORY_PACKAGE_ENTRYPOINT;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ifccad-package-navigation-{name}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture(name: &str) -> PathBuf {
        bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join(name)
    }

    #[test]
    fn navigates_the_minimal_validated_package_without_revalidating_json() {
        let outcome = load_directory_package(fixture("minimal-no-preservation")).unwrap();
        let package = outcome.validated_package.as_ref().expect("strict proof");

        assert_eq!(package.drawing_sets().count(), 1);
        assert_eq!(package.drawings().count(), 1);
        assert_eq!(package.layouts().count(), 1);
        assert_eq!(package.geometry_representations().count(), 1);
        assert!(package.ifcx_node("missing").is_none());
        assert!(package.geometry_representation("missing").is_none());
        assert!(package.ifcdr_resource("missing.ifcdr.json").is_none());

        let set = package.drawing_sets().next().unwrap();
        assert_eq!(set.path(), "drawing-set-main");
        let drawing = set.drawings().next().unwrap();
        assert_eq!(drawing.path(), "drawing-main");
        assert_eq!(
            drawing.representation().path(),
            "representation-modelspace-main"
        );
        assert_eq!(drawing.layouts().count(), 1);

        let layout = drawing.layouts().next().unwrap();
        assert_eq!(layout.path(), "drawing-main-layout-model");
        assert_eq!(layout.name(), "Model");
        assert_eq!(layout.kind(), DrawingLayoutKind::Model);
        assert_eq!(layout.scope().name(), "ModelSpace");

        let representation = layout.representation();
        assert_eq!(representation.role(), "modelspace");
        assert_eq!(representation.uri(), "drawing.ifcdr.json");
        assert_eq!(
            representation.resource().resource_id().as_str(),
            "geometry-modelspace-main"
        );
        let layer = representation.layer(LayerId::new(1)).unwrap();
        assert_eq!(layer.path(), "layer-a-wall");
        assert_eq!(layer.appearance().unwrap().path(), "appearance-dashed-red");
        let appearance = representation.appearance(AppearanceId::new(2)).unwrap();
        assert_eq!(
            appearance.ifcx_definition().unwrap().path(),
            "appearance-default-solid"
        );
    }

    #[test]
    fn navigation_supports_zero_and_unused_or_multiple_representations() {
        let empty_root = TestDirectory::new("empty");
        let mut empty = serde_json::from_slice::<Value>(
            &fs::read(fixture("minimal-no-preservation").join(DIRECTORY_PACKAGE_ENTRYPOINT))
                .unwrap(),
        )
        .unwrap();
        empty["data"] = serde_json::json!([]);
        fs::write(
            empty_root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&empty).unwrap(),
        )
        .unwrap();
        let empty_outcome = load_directory_package(empty_root.path()).unwrap();
        let empty_package = empty_outcome
            .validated_package
            .as_ref()
            .expect("empty proof");
        assert_eq!(empty_package.drawings().count(), 0);
        assert_eq!(empty_package.geometry_representations().count(), 0);

        let multiple_root = TestDirectory::new("multiple");
        let source = fixture("minimal-no-preservation");
        fs::copy(
            source.join("drawing.ifcdr.json"),
            multiple_root.path().join("drawing.ifcdr.json"),
        )
        .unwrap();
        let mut multiple_entrypoint = serde_json::from_slice::<Value>(
            &fs::read(source.join(DIRECTORY_PACKAGE_ENTRYPOINT)).unwrap(),
        )
        .unwrap();
        let mut second_drawing = multiple_entrypoint["data"][1].clone();
        second_drawing["path"] = serde_json::json!("drawing-second");
        let mut unused_representation = multiple_entrypoint["data"][3].clone();
        unused_representation["path"] = serde_json::json!("representation-unused");
        multiple_entrypoint["data"][0]["children"]["Drawings"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("drawing-second"));
        multiple_entrypoint["data"]
            .as_array_mut()
            .unwrap()
            .extend([second_drawing, unused_representation]);
        fs::write(
            multiple_root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&multiple_entrypoint).unwrap(),
        )
        .unwrap();
        let multiple = load_directory_package(multiple_root.path()).unwrap();
        let multiple = multiple.validated_package.as_ref().expect("multiple proof");
        assert_eq!(multiple.drawings().count(), 2);
        assert_eq!(multiple.geometry_representations().count(), 2);
        assert!(multiple
            .geometry_representation("representation-unused")
            .is_some());
    }
}
