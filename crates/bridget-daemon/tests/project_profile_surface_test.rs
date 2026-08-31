const MCP_SOURCE: &str = include_str!("../src/mcp.rs");
const UI_SOURCE: &str = include_str!("../src/ui.rs");

#[test]
fn spec_067_aucune_approbation_ou_rotation_n_est_exposee_par_ui_ou_mcp() {
    for source in [MCP_SOURCE, UI_SOURCE] {
        assert!(
            !source.contains("ProjectProfileApproval"),
            "une approbation projet ne doit pas avoir de surface distante"
        );
        assert!(!source.contains("project_profile_approve"));
        assert!(!source.contains("project_profile_rotate"));
        assert!(!source.contains("project_profile_revoke"));
    }
}
