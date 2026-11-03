Standalone upstream ClipperLib 6.2.6 reference harness for parity bisection.

Build (needs Eigen + a oneapi/tbb/scalable_allocator.h stub — provided here):
  nix-build '<nixpkgs>' -A eigen --no-out-link   # -> $EINC
  g++ -I$EINC -IOrcaSlicer/deps_src/clipper \
      -IOrcaSlicer/src -I tools/clipref \
      -c OrcaSlicer/deps_src/clipper/clipper.cpp -o clipper.o
  g++ -I$EINC -I tools/clipref -I ...deps_src/clipper clipin.cpp clipper.o -o clipin
  g++ -I$EINC -I tools/clipref -I ...deps_src/clipper dp.cpp clipper.o -o dp

clipin: reads "x,y x,y ..." lines tagged P (open subject) / C (closed clip) on
stdin, runs Clipper ctIntersection NonZero/NonZero, prints each output polyline.
dp: reads a scaled-int polyline, prints upstream MultiPoint::_douglas_peucker
result at tolerance scaled(0.012)=12000.

Used to prove ares `intersection_open_polylines` + `simplify_linear_points`
are byte-identical to upstream on the bottom-hilbert fragment; the residual
vertex-drop is downstream of both (entity segmentation / emission).
