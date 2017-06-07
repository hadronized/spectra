use std::collections::{HashMap, HashSet};

/// A shader module.
///
/// A shader module is a container that associates some shading code to several identifiers.
struct ShaderModule {
  symbols: HashMap<Identifier, ShadingCode>
}

/// Spectra Shading Language AST.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SSL {
  /// An `export list_of_identifiers_` statement.
  Export(ExportList),
  /// A `from module use list of identifiers` statement.
  FromUse(ImportList),
  /// A `pipeline { list_of_pipeline_attributes }` statement.
  Pipeline(PipelineStatement),
  /// A yield statement, valid in geometry shaders.
  Yield(GeometryYieldExpression),
}

/// A module.
type Module = String;
/// An identifier.
type Identifier = String;
/// Some opaque shading code.
type ShadingCode = String;
/// An expression.
type Expression = String;

/// An export non-empty list.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ExportList {
  list: HashSet<Identifier>
}

/// An import non-empty list.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportList {
  module: Module,
  list: HashSet<Identifier>
}

/// A pipeline statement.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PipelineStatement {
  attributes: Vec<PipelineAttribute>
}

/// Attributes that can be set in a pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PipelineAttribute {
  /// Maximum vertices that the geometry shader can output.
  GeometryShaderMaxVertices(u32),
  /// Number of times the geometry shader must be invoked.
  GeometryShaderInvokations(u32)
}

/// Expressions that can be yielded in a geometry shader.
#[derive(Clone, Debug, Eq, PartialEq)]
enum GeometryYieldExpression {
  /// Yield a primitive.
  YieldPrimitive,
  /// Yield a primitive’s vertex (fold vertex).
  YieldFoldVertex(Expression)
}

/// Error that can occur when parsing SSL code.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ParseError {
  ExpressionError(String)
}
 
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
