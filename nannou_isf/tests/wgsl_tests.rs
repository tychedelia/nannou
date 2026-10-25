//! Tests for translating ISF fragment shaders to WGSL with [`nannou_isf::wgsl_from_isf`].

use nannou_isf::{IsfAssetLoaderError, wgsl_from_isf};

/// The multi-pass ISF shader loaded by the `isf_simple` example.
const MULTI_PASS_ISF: &str = include_str!("../../examples/assets/isf/Test-MultiPassRendering.fs");

#[test]
fn test_multi_pass_isf_translates_to_wgsl() {
    let isf = isf::parse(MULTI_PASS_ISF).unwrap();
    let wgsl = wgsl_from_isf(&isf, MULTI_PASS_ISF, "Test-MultiPassRendering.fs").unwrap();
    assert!(wgsl.contains("@fragment"));
    assert!(wgsl.contains("fn main(@location(0) isf_FragNormCoord: vec2<f32>)"));
    assert!(wgsl.contains("var bufferVariableNameA: texture_2d<f32>;"));
}

#[test]
fn test_invalid_glsl_reports_path() {
    let src = "/*{ \"INPUTS\": [] }*/\nvoid main() { gl_FragColor = undefined_value; }";
    let isf = isf::parse(src).unwrap();
    match wgsl_from_isf(&isf, src, "invalid.fs") {
        Err(IsfAssetLoaderError::Glsl(diagnostics)) => {
            assert!(diagnostics.contains("invalid.fs"));
            assert!(diagnostics.contains("undefined_value"));
        }
        other => panic!("expected a GLSL parse error, got {other:?}"),
    }
}
