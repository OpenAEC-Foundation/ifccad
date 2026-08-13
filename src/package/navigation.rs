// Typed package navigation remains crate-internal until conversion experiments
// establish the stable consumer-facing surface.
#![allow(dead_code)]

use super::analysis::ValidatedIfccadPackage;
use crate::ifcdr::{AppearanceId, LayerId, ScopeRef, ValidatedIfcdrResource};
use serde_json::Value;

impl ValidatedIfccadPackage {
    pub(crate) fn drawing_sets(&self) -> impl Iterator<Item = DrawingSetRef<'_>> {
        typed_nodes(self, "openaec:DrawingSet").map(|node_index| DrawingSetRef {
            package: self,
            node_index,
        })
    }

    pub(crate) fn drawings(&self) -> impl Iterator<Item = DrawingRef<'_>> {
        typed_nodes(self, "openaec:Drawing").map(|node_index| DrawingRef {
            package: self,
            node_index,
        })
    }

    pub(crate) fn layouts(&self) -> impl Iterator<Item = DrawingLayoutRef<'_>> {
        typed_nodes(self, "openaec:DrawingLayout").map(|node_index| DrawingLayoutRef {
            package: self,
            node_index,
        })
    }

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

    fn layer(&self, path: &str) -> Option<LayerRef<'_>> {
        self.typed_node(path, "openaec:Layer")
            .map(|node_index| LayerRef {
                package: self,
                node_index,
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
        pub(crate) struct $name<'a> {
            package: &'a ValidatedIfccadPackage,
            node_index: usize,
        }

        impl<'a> $name<'a> {
            fn node(&self) -> &'a Value {
                &nodes(self.package)[self.node_index]
            }

            pub(crate) fn path(&self) -> &str {
                self.node()["path"].as_str().expect("validated node path")
            }
        }
    };
}

node_ref!(DrawingSetRef);
node_ref!(DrawingRef);
node_ref!(DrawingLayoutRef);
node_ref!(GeometryRepresentationRef);
node_ref!(LayerRef);
node_ref!(AppearanceRef);

impl<'a> DrawingSetRef<'a> {
    pub(crate) fn drawings(&self) -> impl Iterator<Item = DrawingRef<'a>> + 'a {
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
    pub(crate) fn representation(&self) -> Option<GeometryRepresentationRef<'a>> {
        self.node()
            .pointer("/children/Representation")
            .and_then(Value::as_str)
            .and_then(|path| self.package.geometry_representation(path))
    }

    pub(crate) fn layouts(&self) -> impl Iterator<Item = DrawingLayoutRef<'a>> + 'a {
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
    pub(crate) fn name(&self) -> &str {
        self.node()
            .pointer("/attributes/name")
            .and_then(Value::as_str)
            .expect("validated layout name")
    }

    pub(crate) fn kind(&self) -> &str {
        self.node()
            .pointer("/attributes/kind")
            .and_then(Value::as_str)
            .expect("validated layout kind")
    }

    pub(crate) fn representation(&self) -> Option<GeometryRepresentationRef<'a>> {
        self.package.geometry_representation(
            &self
                .package
                .evidence()
                .bindings
                .layout_by_path
                .get(self.path())?
                .representation_path,
        )
    }

    pub(crate) fn scope(&self) -> Option<ScopeRef<'_>> {
        let binding = self
            .package
            .evidence()
            .bindings
            .layout_by_path
            .get(self.path())?;
        self.package
            .ifcdr_resource(&binding.ifcdr_uri)?
            .scope(binding.scope_id)
    }
}

impl<'a> GeometryRepresentationRef<'a> {
    pub(crate) fn role(&self) -> &str {
        self.node()
            .pointer("/attributes/geometry/role")
            .and_then(Value::as_str)
            .expect("validated representation role")
    }

    pub(crate) fn uri(&self) -> &str {
        self.node()
            .pointer("/attributes/geometry/uri")
            .and_then(Value::as_str)
            .expect("validated representation URI")
    }

    pub(crate) fn resource(&self) -> &'a ValidatedIfcdrResource {
        self.package.evidence().bindings.geometry_ifcdr_by_path[self.path()].as_ref()
    }

    pub(crate) fn layer(&self, id: LayerId) -> Option<LayerRef<'a>> {
        let path = self
            .package
            .evidence()
            .bindings
            .ifcx_layer_by_ifcdr_id
            .get(&(self.uri().to_owned(), id))?;
        self.package.layer(path)
    }

    pub(crate) fn appearance(&self, id: AppearanceId) -> Option<AppearanceRef<'a>> {
        let path = self
            .package
            .evidence()
            .bindings
            .ifcx_appearance_by_ifcdr_id
            .get(&(self.uri().to_owned(), id))?;
        self.package.appearance(path)
    }
}

impl<'a> LayerRef<'a> {
    pub(crate) fn appearance(&self) -> Option<AppearanceRef<'a>> {
        self.node()
            .pointer("/attributes/appearance")
            .and_then(Value::as_str)
            .and_then(|path| self.package.appearance(path))
    }
}

impl AppearanceRef<'_> {
    pub(crate) fn value(&self) -> &Value {
        self.node()
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
            drawing.representation().unwrap().path(),
            "representation-modelspace-main"
        );
        assert_eq!(drawing.layouts().count(), 1);

        let layout = drawing.layouts().next().unwrap();
        assert_eq!(layout.path(), "drawing-main-layout-model");
        assert_eq!(layout.name(), "Model");
        assert_eq!(layout.kind(), "model");
        assert_eq!(layout.scope().unwrap().name(), "ModelSpace");

        let representation = layout.representation().unwrap();
        assert_eq!(representation.role(), "modelspace");
        assert_eq!(representation.uri(), "drawing.ifcdr.json");
        assert_eq!(
            representation.resource().header().resource_id(),
            "geometry-modelspace-main"
        );
        let layer = representation.layer(LayerId::new(1)).unwrap();
        assert_eq!(layer.path(), "layer-a-wall");
        assert_eq!(layer.appearance().unwrap().path(), "appearance-dashed-red");
        let appearance = representation.appearance(AppearanceId::new(2)).unwrap();
        assert_eq!(appearance.path(), "appearance-default-solid");
        assert_eq!(appearance.value()["type"], "openaec:Appearance");
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
