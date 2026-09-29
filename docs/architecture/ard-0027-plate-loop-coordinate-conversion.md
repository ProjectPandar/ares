# ARD-0027: Plate loops translate before unscaling

Upstream emits skirt/brim integer paths with zero origin. Reconstruct
those plate coordinates before floating-point conversion at the emission
boundary. Unscaling a local point and then adding the model origin can
cross a half-millimeter-digit rounding threshold even when the two
operations are algebraically equal.

Internal cursors, wipe source points and geometry consumers remain in
the current local representation. Other roles retain their upstream
model-space conversion order. Do not alter formatting or accept a
coordinate tolerance to hide the difference.
