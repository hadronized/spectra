//! The Cheddar shading language parser.
//!
//! This module exports several identifiers, parser function combinators and utility functions to
//! transform and handle the Cheddar AST.
use nom::alphanumeric;
use std::str::from_utf8_unchecked;

use glsl::parser::{external_declaration, identifier};
pub use glsl::parser::{ParseError, ParseResult, parse, parse_str};
use render::shader::cheddar::syntax;

// Turn a &[u8] into a String.
#[inline]
fn bytes_to_string(bytes: &[u8]) -> String {
  unsafe { from_utf8_unchecked(bytes).to_owned() }
}

/// Parse a module separator and a module name.
named!(module_sep_n_name,
  do_parse!(
    char!('.') >>
    name: alphanumeric >>
    (name)
  )
);

/// Parse a module path.
///
/// foo
/// foo.bar
/// foo.bar.zoo
named!(pub module_path<&[u8], syntax::ModulePath>,
  do_parse!(
    // recognize at least one module name
    base: identifier >>
    // recognize the rest of the path, if any
    rest: many0!(module_sep_n_name) >>

    ({
      let mut rest = rest.clone(); // FIXME: meh?
      rest.insert(0, base.as_bytes());

      syntax::ModulePath {
        path: rest.into_iter().map(bytes_to_string).collect()
      }
    })
  )
);

/// Parse a symbol list.
///
///     ( item0, item1, item2, …)
named!(pub symbol_list<&[u8], Vec<syntax::ModuleSymbol>>,
  ws!(
    delimited!(char!('('),
               separated_list!(char!(','), identifier),
               char!(')')
    )
  )
);

/// Parse an import list.
named!(pub import_list<&[u8], syntax::ImportList>,
  ws!(do_parse!(
    tag!("use") >>
    from_module: module_path >>
    symbols: symbol_list >>
    (syntax::ImportList { module: from_module, list: symbols })
  ))
);

/// Parse a module.
named!(pub module<&[u8], syntax::Module>,
  ws!(do_parse!(
    imports: many0!(import_list) >>
    glsl: many0!(external_declaration) >>
    (syntax::Module { imports, glsl })
  ))
);

#[cfg(test)]
mod tests {
  use nom::IResult;
  use super::*;

  #[test]
  fn parse_module_sep_n_name() {
    assert_eq!(module_sep_n_name(&b".foo"[..]), IResult::Done(&b""[..], &b"foo"[..]));
    assert_eq!(module_sep_n_name(&b".foo.bar"[..]), IResult::Done(&b".bar"[..], &b"foo"[..]));
  }
  
  #[test]
  fn parse_module_path_simple() {
    assert_eq!(module_path(&b"foo"[..]), IResult::Done(&b""[..], syntax::ModulePath { path: vec!["foo".into()] }));
    assert_eq!(module_path(&b"  \n\tfoo \n"[..]), IResult::Done(&b""[..], syntax::ModulePath { path: vec!["foo".into()] }));
  }
  
  #[test]
  fn parse_module_path_several() {
    assert_eq!(module_path(&b"foo.bar.zoo"[..]), IResult::Done(&b""[..], syntax::ModulePath { path: vec!["foo".into(), "bar".into(), "zoo".into()] }));
  }
  
  #[test]
  fn parse_symbol_list() {
    let foo = "foo".to_owned();
    let bar = "bar".to_owned();
    let zoo = "zoo".to_owned();
    let list = vec![foo, bar, zoo];
  
    assert_eq!(symbol_list(&b"(foo,bar,zoo)"[..]), IResult::Done(&b""[..], list.clone()));
    assert_eq!(symbol_list(&b" ( foo,bar,zoo  ) "[..]), IResult::Done(&b""[..], list.clone()));
    assert_eq!(symbol_list(&b"( foo, bar ,   zoo  )"[..]), IResult::Done(&b""[..], list.clone()));
  }
  
  #[test]
  fn parse_import_list() {
    let foo = "foo".to_owned();
    let bar = "bar".to_owned();
    let zoo_woo = syntax::ModulePath { path: vec!["zoo".into(), "woo".into()] };
    let expected = syntax::ImportList { module: zoo_woo, list: vec![foo, bar] };
  
    assert_eq!(import_list(&b"use zoo.woo (foo, bar)"[..]), IResult::Done(&b""[..], expected.clone()));
    assert_eq!(import_list(&b" use    zoo.woo    (   foo  ,   bar  )"[..]), IResult::Done(&b""[..], expected.clone()));
    assert_eq!(import_list(&b"use zoo.woo (foo,\nbar)"[..]), IResult::Done(&b""[..], expected.clone()));
  }
}
