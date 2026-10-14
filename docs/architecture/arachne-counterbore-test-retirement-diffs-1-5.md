# Counterbore test-retirement exact diffs — cases 1–5

**PROPOSAL ONLY — not applied.** See the [request](arachne-counterbore-test-retirement-request.md).
Exact preimage: `2e56c73756e014f8c1539f5e8e9bc175e3da96a5`.

## Case 1: `arachne_sacrificial_layer_counterbore_hole_remains_out_of_scope`

Retire the obsolete classic-preflight Arachne Filled rejection. The other three positive preflight tests remain byte-for-byte. Remove only its now-unused SliceError import.

```diff
--- a/crates/ares-core/src/project_slice/tests/perimeters/classic/preflight.rs
+++ b/crates/ares-core/src/project_slice/tests/perimeters/classic/preflight.rs
@@ -1,3 +1,3 @@
-use crate::{SliceError, slice_project};
+use crate::slice_project;

 use super::super::super::support::{KsrArchive, metadata};
@@ -19,25 +19,4 @@
 }

-#[tokio::test]
-async fn arachne_sacrificial_layer_counterbore_hole_remains_out_of_scope() {
-    let mut archive = KsrArchive::new();
-    archive.replace_unique(
-        "Metadata/project_settings.config",
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-    );
-    archive.replace_unique(
-        "Metadata/project_settings.config",
-        "\"wall_generator\": \"classic\"",
-        "\"wall_generator\": \"arachne\"",
-    );
-    assert_eq!(
-        slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-            .await
-            .unwrap_err(),
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
-    );
-}
-
 #[tokio::test]
 async fn alternate_extra_wall_adds_one_wall_on_the_second_layer() {
```

## Case 2: `task22o16_counterbore_preflight_precedes_layer_region_lifecycle`

Retire the zero-finish rejection; replace with supported completion. Same KsrArchive, sacrificiallayer/Filled alias and Arachne overrides. Rename the test, require successful nonempty output and finish_invocations()==1. perimeters.rs:180 calls finish_with_arachne once per prepared project; layer_region.rs:42 increments once. This preserves the existing hook without changing its production owner or inventing a new error. Remove unused SliceError import; CONFIG and setup remain.

```diff
--- a/crates/ares-core/src/project_slice/tests/perimeters/layer_region/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/perimeters/layer_region/lifecycle.rs
@@ -1,3 +1,3 @@
-use crate::{SliceError, project_slice::perimeters::layer_region, slice_project};
+use crate::{project_slice::perimeters::layer_region, slice_project};

 use super::super::super::support::{KsrArchive, metadata};
@@ -6,5 +6,5 @@

 #[tokio::test]
-async fn task22o16_counterbore_preflight_precedes_layer_region_lifecycle() {
+async fn arachne_filled_counterbore_reaches_layer_region_lifecycle() {
     let mut archive = KsrArchive::new();
     archive.replace_unique(
@@ -19,10 +19,8 @@
     );
     layer_region::reset_finish_invocations();
-    assert_eq!(
-        slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-            .await
-            .unwrap_err(),
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned())
-    );
-    assert_eq!(layer_region::finish_invocations(), 0);
+    let output = slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
+        .await
+        .unwrap();
+    assert!(!output.is_empty());
+    assert_eq!(layer_region::finish_invocations(), 1);
 }
```

## Case 3: `task22o18_earlier_errors_leave_invocations_zero`

Retire only the obsolete Filled for-loop, not this mixed test. Keep task22o18_earlier_errors_leave_invocations_zero and its independent injected TopSafetyDifference error, InvalidInput assertion, fill_surfaces::invocations()==0 and hook resets verbatim.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/fill_surfaces/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/fill_surfaces/lifecycle.rs
@@ -6,26 +6,4 @@
 #[tokio::test]
 async fn task22o18_earlier_errors_leave_invocations_zero() {
-    for (from, to, error) in [(
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-        "counterbore_hole_bridging",
-    )] {
-        let mut archive = KsrArchive::new();
-        archive.replace_unique("Metadata/project_settings.config", from, to);
-        archive.replace_unique(
-            "Metadata/project_settings.config",
-            "\"wall_generator\": \"classic\"",
-            "\"wall_generator\": \"arachne\"",
-        );
-        fill_surfaces::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            SliceError::UnsupportedProjectFeature(error.to_owned())
-        );
-        assert_eq!(fill_surfaces::invocations(), 0);
-    }
-
     fill_surfaces::reset_invocations();
     surface_type_detection::reset_geometry_hooks();
```

## Case 4: `task22o25_every_earlier_capability_error_precedes_promotion`

Retire only the counterbore capability-precedence test. The separate earlier-geometry-stage assertions, public parse error, promotion invocation1/commit0/disposal1 and propagation0 test, plus assert_precedes helper remain unchanged. The removed disposal0 belongs only to the now-supported Filled input.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/horizontal_shell_promotion/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/horizontal_shell_promotion/lifecycle.rs
@@ -93,32 +93,4 @@
 }

-#[tokio::test]
-async fn task22o25_every_earlier_capability_error_precedes_promotion() {
-    for (from, to, feature) in [(
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-        "counterbore_hole_bridging",
-    )] {
-        let mut archive = KsrArchive::new();
-        archive.replace_unique("Metadata/project_settings.config", from, to);
-        archive.replace_unique(
-            "Metadata/project_settings.config",
-            "\"wall_generator\": \"classic\"",
-            "\"wall_generator\": \"arachne\"",
-        );
-        horizontal_shell_promotion::reset_hooks();
-        horizontal_shell_propagation::reset_hooks();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            SliceError::UnsupportedProjectFeature(feature.to_owned())
-        );
-        assert_eq!(horizontal_shell_promotion::invocations(), 0);
-        assert_eq!(horizontal_shell_promotion::disposals(), 0);
-        assert_eq!(horizontal_shell_propagation::invocations(), 0);
-    }
-}
-
 async fn assert_precedes(expected: &str) {
     horizontal_shell_promotion::reset_hooks();
```

## Case 5: `task22o17_counterbore_fails_before_detection`

Retire the obsolete detection rejection and empty module. This file contains only the obsolete test; delete it and its parent mod lifecycle registration rather than leaving an empty module. Detection options/transaction/ownership/metamorphic and their hooks/assertions remain unchanged; options.rs still uses detection invocations/reset_invocations.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/surface_type_detection/lifecycle.rs
+++ /dev/null
@@ -1,29 +0,0 @@
-use crate::{SliceError, slice_project};
-
-use super::super::super::support::{KsrArchive, metadata};
-use crate::project_slice::prepare_infill::surface_type_detection::{
-    invocations, reset_invocations,
-};
-
-#[tokio::test]
-async fn task22o17_counterbore_fails_before_detection() {
-    let mut archive = KsrArchive::new();
-    archive.replace_unique(
-        "Metadata/project_settings.config",
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-    );
-    archive.replace_unique(
-        "Metadata/project_settings.config",
-        "\"wall_generator\": \"classic\"",
-        "\"wall_generator\": \"arachne\"",
-    );
-    reset_invocations();
-    assert_eq!(
-        slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-            .await
-            .unwrap_err(),
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned())
-    );
-    assert_eq!(invocations(), 0);
-}
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/surface_type_detection.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/surface_type_detection.rs
@@ -2,5 +2,4 @@
 mod fixture;
 mod ksr;
-mod lifecycle;
 mod metamorphic;
 mod options;
```
