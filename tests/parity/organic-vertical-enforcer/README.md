# Vertical support painting

Each 3MF contains the same synthetic closed T: a 5 × 5 mm pillar from Z=0 to
Z=5 mm, and a 15 × 5 mm cap from Z=5 to Z=7.5 mm. They meet along the full top
of the pillar. In the component mesh, the pillar spans X and Y from −2.5 to
2.5 mm; its low-X vertical face has two triangles with `paint_supports="4"`
(an unsplit enforcer). No other faces are painted, and there are no annotation
volumes. Settings use the X1 Carbon 0.4 mm nozzle, the 0.20 mm Standard
process, Bambu PLA Basic, and manual tree support. The only difference between
packages is `support_style` in `Metadata/project_settings.config`:
`organic`, `default`, or `tree_slim`.
