# Narrow Qt hull inspection

`narrow-fan.stl` is synthetic: a triangle fan with distinct vertices and a thickness near Qhull's roundoff threshold. Its local point order makes Qt triangulate adjacent polygons into two facets with identical vertex sets. They remain separate facets in the ordered hull. The ordinary open box in the inspection tests supplies a non-narrow control. The tests use live reference output; no inspection JSON from a reference build is stored here.
