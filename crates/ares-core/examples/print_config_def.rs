//! Dump the generated PrintConfigDef JSON export to stdout.
//! Used by scripts/verify-configdef.mjs for the independent per-key audit.

fn main() {
    std::io::Write::write_all(&mut std::io::stdout(), ares_core::print_config_def_json())
        .expect("stdout");
}
