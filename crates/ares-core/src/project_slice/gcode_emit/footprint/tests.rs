use super::sanitize_instance_name;

#[test]
fn sanitize_matches_upstream_character_class() {
    assert_eq!(sanitize_instance_name("cube10.stl"), "cube10.stl");
    assert_eq!(sanitize_instance_name("a b!!c.stl"), "a_b_c.stl");
    assert_eq!(sanitize_instance_name("!!lead"), "lead");
    assert_eq!(sanitize_instance_name("trail!!"), "trail");
    assert_eq!(sanitize_instance_name("a:b,c\"d'e"), "a_b_c_d_e");
    assert_eq!(sanitize_instance_name("hyphen.dot/kept"), "hyphen.dot/kept");
    assert_eq!(sanitize_instance_name("__foo__"), "_foo_");
}
