//! Shading language.
//!
//! This is an augmented GLSL that supports modules and deep analysis. That last feature makes it
//! possible to define a shader pipeline by composing several modules and use them in a single
//! module without even declaring a *vertex shader* or *fragment shader*: everything is analyzed and
//! shader stages are created from GLSL fragments.
//!
//! The `parser` module exports [nom](https://crates.io/crates/nom) parsers and `syntax` gives the
//! parsed AST.
pub mod parser;
pub mod syntax;

// // FIXME: move that into a specific module
// #[test]
// fn parse_pipeline_attribute() {
//   let geo_max_vertices  = "geometry_shader_max_vertices = 3";
//   let geo_max_vertices1 = "geometry_shader_max_vertices =3";
//   let geo_max_vertices2 = "geometry_shader_max_vertices =";
//   let geo_max_vertices3 = "geometry_shader_max_vertices = ";
//   let geo_invokations  = "geometry_shader_invokations = 1";
//   let geo_invokations1 = "geometry_shader_invokations =1";
//   let geo_invokations2 = "geometry_shader_invokations =";
//   let geo_invokations3 = "geometry_shader_invokations = ";
// 
//   assert_eq!(geo_max_vertices.parse::<PipelineAttribute>(), Ok(PipelineAttribute::GeometryShaderMaxVertices(3)));
//   assert_eq!(geo_max_vertices1.parse::<PipelineAttribute>(), Ok(PipelineAttribute::GeometryShaderMaxVertices(3)));
//   assert!(geo_max_vertices2.parse::<PipelineAttribute>().is_err());
//   assert!(geo_max_vertices3.parse::<PipelineAttribute>().is_err());
//   assert_eq!(geo_invokations.parse::<PipelineAttribute>(), Ok(PipelineAttribute::GeometryShaderInvokations(1)));
//   assert_eq!(geo_invokations1.parse::<PipelineAttribute>(), Ok(PipelineAttribute::GeometryShaderInvokations(1)));
//   assert!(geo_invokations2.parse::<PipelineAttribute>().is_err());
//   assert!(geo_invokations3.parse::<PipelineAttribute>().is_err());
// }
// 
// // FIXME: move that into a specific module
// #[test]
// fn parse_geometry_yield_expression() {
//   let yieldprim = "yieldprim";
//   let yield_1 = "yield FoldVertex(vertex[i].color)";
// 
//   assert_eq!(yieldprim.parse::<GeometryYieldExpression>(), Ok(GeometryYieldExpression::YieldPrimitive));
//   assert_eq!(yield_1.parse::<GeometryYieldExpression>(), Ok(GeometryYieldExpression::YieldFoldVertex("FoldVertex(vertex[i].color)".into())));
// }
