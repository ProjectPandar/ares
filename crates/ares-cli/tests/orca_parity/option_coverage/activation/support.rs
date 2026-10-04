use serde_json::{Map, Value, json};

pub(super) fn activate(key: &str, value: Option<&Value>, process: &mut Map<String, Value>) {
    if key == "support_style" {
        // `SupportParameters.hpp:190–204`: Tree-only styles on Normal
        // normalize to Grid instead of activating the requested generator.
        let tree = matches!(
            value.unwrap().as_str().unwrap(),
            "tree_slim" | "tree_strong" | "tree_hybrid" | "organic"
        );
        process.insert("enable_support".into(), json!("1"));
        process.insert(
            "support_type".into(),
            json!(if tree { "tree(auto)" } else { "normal(auto)" }),
        );
        return;
    }
    let style = match key {
        // `TreeSupport.cpp::drop_nodes/generate_contact_points/draw_circles`.
        "tree_support_auto_brim"
        | "tree_support_branch_angle"
        | "tree_support_branch_diameter"
        | "tree_support_branch_distance"
        | "tree_support_brim_width" => "tree_slim",
        // `TreeSupportCommon.hpp:83–92`; shared controls use this owner too.
        "tree_support_angle_slow"
        | "tree_support_branch_angle_organic"
        | "tree_support_branch_diameter_angle"
        | "tree_support_branch_diameter_organic"
        | "tree_support_branch_distance_organic"
        | "tree_support_top_rate"
        | "tree_support_wall_count" => "organic",
        // Raw Grid normalizes to Organic, but unlike explicit Organic
        // passes schema tip values above branch diameter (Print.cpp:1580–1610).
        "tree_support_tip_diameter" => "grid",
        _ => return,
    };
    process.insert("enable_support".into(), json!("1"));
    process.insert("support_type".into(), json!("tree(auto)"));
    process.insert("support_style".into(), json!(style));
    match key {
        // `TreeSupport.cpp:3422`: don't mask the legal distance minimum1.
        "tree_support_branch_distance" => {
            process.insert("max_bridge_length".into(), json!("2"));
        }
        "tree_support_angle_slow" => {
            process.insert("tree_support_branch_angle_organic".into(), json!("60"));
        }
        "tree_support_tip_diameter" => {
            process.insert("tree_support_branch_diameter_organic".into(), json!("10"));
            process.insert("support_line_width".into(), json!("0.1"));
            process.insert("layer_height".into(), json!("0.08"));
            process.insert("initial_layer_print_height".into(), json!("0.08"));
        }
        // `TreeSupport.cpp:2135–2136`: first object layer, no raft.
        "tree_support_auto_brim" | "tree_support_brim_width" => {
            process.insert("raft_layers".into(), json!("0"));
            if key == "tree_support_brim_width" {
                process.insert("tree_support_auto_brim".into(), json!("0"));
            }
        }
        _ => {}
    }
}
