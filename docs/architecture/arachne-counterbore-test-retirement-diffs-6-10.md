# Counterbore test-retirement exact diffs — cases 6–10

**PROPOSAL ONLY — not applied.** See the [request](arachne-counterbore-test-retirement-request.md).
Exact preimage: `2e56c73756e014f8c1539f5e8e9bc175e3da96a5`.

## Case 6: `task22o23_all_earlier_capability_errors_keep_filtering_uninvoked`

Retire only obsolete filtering capability precedence. Keep all geometry-stage/anchor failure zero-invocation assertions and assert_precedes helper verbatim. Remove the now-unused SliceError import, nothing else.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_filtering/lifecycle/precedence.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_filtering/lifecycle/precedence.rs
@@ -1,4 +1,3 @@
 use crate::{
-    SliceError,
     project_slice::{
         prepare_infill::{
@@ -12,29 +11,4 @@

 #[tokio::test]
-async fn task22o23_all_earlier_capability_errors_keep_filtering_uninvoked() {
-    for (from, to, expected) in [(
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
-        vertical_shell_filtering::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            SliceError::UnsupportedProjectFeature(expected.to_owned())
-        );
-        assert_eq!(vertical_shell_filtering::invocations(), 0);
-    }
-}
-
-#[tokio::test]
 async fn task22o23_every_earlier_geometry_stage_precedes_filtering() {
     surface_type_detection::reset_geometry_hooks();
```

## Case 7: `task22o20_earlier_errors_have_precedence_and_zero_invocations`

Retire only obsolete projection capability precedence. Keep O17/O19 error identity, zero invocations, later-slot transactional failure event count1835, final event, predecessor drop probe and reset assertions unchanged.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_projection/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_projection/lifecycle.rs
@@ -12,29 +12,4 @@

 #[tokio::test]
-async fn task22o20_earlier_errors_have_precedence_and_zero_invocations() {
-    for (from, to, expected) in [(
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
-    )] {
-        let mut archive = KsrArchive::new();
-        archive.replace_unique("Metadata/project_settings.config", from, to);
-        archive.replace_unique(
-            "Metadata/project_settings.config",
-            "\"wall_generator\": \"classic\"",
-            "\"wall_generator\": \"arachne\"",
-        );
-        vertical_shell_projection::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            expected
-        );
-        assert_eq!(vertical_shell_projection::invocations(), 0);
-    }
-}
-
-#[tokio::test]
 async fn task22o20_o17_and_o19_geometry_failures_precede_projection() {
     surface_type_detection::reset_geometry_hooks();
```

## Case 8: `task22o22_earlier_capability_errors_have_precedence`

Retire only obsolete regularization capability precedence. Keep inactive typed-mode/no-geometry behavior, all predecessor/anchor failure zero-invocation assertions and their original error strings unchanged.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_regularization/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_regularization/lifecycle.rs
@@ -31,29 +31,4 @@

 #[tokio::test]
-async fn task22o22_earlier_capability_errors_have_precedence() {
-    for (from, to, expected) in [(
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
-    )] {
-        let mut archive = KsrArchive::new();
-        archive.replace_unique("Metadata/project_settings.config", from, to);
-        archive.replace_unique(
-            "Metadata/project_settings.config",
-            "\"wall_generator\": \"classic\"",
-            "\"wall_generator\": \"arachne\"",
-        );
-        vertical_shell_regularization::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            expected
-        );
-        assert_eq!(vertical_shell_regularization::invocations(), 0);
-    }
-}
-
-#[tokio::test]
 async fn task22o22_o17_o19_o20_and_o21_failures_precede_regularization() {
     surface_type_detection::reset_geometry_hooks();
```

## Case 9: `task22o21_earlier_capability_errors_have_precedence`

Retire only obsolete trimming capability precedence. Keep O17/O19/O20 and anchor failure assertions; preserve later-object transactional whole-project failure count461, last event, drop probe and resets verbatim.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_trimming/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_trimming/lifecycle.rs
@@ -11,29 +11,4 @@

 #[tokio::test]
-async fn task22o21_earlier_capability_errors_have_precedence() {
-    for (from, to, expected) in [(
-        "\"counterbore_hole_bridging\": \"none\"",
-        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
-        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
-    )] {
-        let mut archive = KsrArchive::new();
-        archive.replace_unique("Metadata/project_settings.config", from, to);
-        archive.replace_unique(
-            "Metadata/project_settings.config",
-            "\"wall_generator\": \"classic\"",
-            "\"wall_generator\": \"arachne\"",
-        );
-        vertical_shell_trimming::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            expected
-        );
-        assert_eq!(vertical_shell_trimming::invocations(), 0);
-    }
-}
-
-#[tokio::test]
 async fn task22o21_o17_o19_and_o20_failures_precede_trimming() {
     surface_type_detection::reset_geometry_hooks();
```

## Case 10: `task22o19_earlier_capability_errors_have_precedence`

Retire only obsolete vertical-shell capability precedence. Keep the existing O17 geometry failure error identity, zero-cache invocation and hook resets verbatim.

```diff
--- a/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shells/lifecycle.rs
+++ b/crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shells/lifecycle.rs
@@ -6,29 +6,4 @@

 #[tokio::test]
-async fn task22o19_earlier_capability_errors_have_precedence() {
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
-        vertical_shells::reset_invocations();
-        assert_eq!(
-            slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
-                .await
-                .unwrap_err(),
-            SliceError::UnsupportedProjectFeature(feature.to_owned())
-        );
-        assert_eq!(vertical_shells::invocations(), 0);
-    }
-}
-
-#[tokio::test]
 async fn task22o19_o17_geometry_failure_precedes_cache_population() {
     surface_type_detection::reset_geometry_hooks();
```
