# support-interface-top-layers-base-connect RED fixtures

Non-P1S stock Creality K1 0.4 mm overhang project (tree(auto) support,
organic style, `support_base_pattern=rectilinear`). The discovery target/control
inputs are preserved byte-for-byte.

- `control.3mf` — fixture defaults, `support_interface_top_layers=2`
  (SHA-256 `7a8b9f2abfb2bbc8b74b76bd9affb458ad5a96c81353cfa767b66b773db23239`);
  native and Ares complete streams agree today and must stay green.
- `top4.3mf` — only `support_interface_top_layers=4` differs
  (SHA-256 `a4e969503b241509a5d8e87f02484b41351d6de1883e5391c064c7967f127380`);
  dual-side probes proved every stage equal (base polygons, fill parameters,
  closing/offset expolygons, pre-connect vertical segments) except one
  post-`connect_base_support` polyline at support layer print_z 0.8.

No Native output is committed; the cases slice Native at test time in both
official and seeded replay modes through `native_stream_case::check`.
