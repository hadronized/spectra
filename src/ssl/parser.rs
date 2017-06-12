use nom::{ErrorKind, IResult, Needed, alphanumeric, anychar};
use std::str::from_utf8_unchecked;

use ssl::syntax;

/// Parse an identifier.
named!(identifier<&[u8], syntax::Identifier>,
  do_parse!(
    name: verify!(take_while1!(identifier_pred), verify_identifier) >>
    (bytes_to_string(name))
  )
);

#[inline]
fn identifier_pred(c: u8) -> bool {
  let ch = char::from(c);
  ch.is_alphanumeric() || ch == '_'
}

#[inline]
fn verify_identifier(s: &[u8]) -> bool {
  !char::from(s[0]).is_digit(10)
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
named!(module_path<&[u8], syntax::ModulePath>,
  do_parse!(
    // recognize at least one module name
    base: alphanumeric >>
    // recognize the rest of the path, if any
    rest: many0!(module_sep_n_name) >>

    ({
      let mut rest = rest.clone(); // FIXME: meh?
      rest.insert(0, base);

      syntax::ModulePath {
        path: rest.into_iter().map(bytes_to_string).collect()
      }
    })
  )
);

/// Parse a module list.
///
///     ( item0, item1, item2, …)
named!(module_list<&[u8], Vec<syntax::ModulePath>>,
  ws!(
    delimited!(char!('('),
               separated_list!(char!(','), module_path),
               char!(')')
    )
  )
);

/// Parse an export list.
named!(export_list<&[u8], syntax::ExportList>,
  ws!(do_parse!(
    tag!("export") >>
    modules: module_list >>

    (
      syntax::ExportList {
        export_list: modules
      }
    )
  ))
);

///// Parse an import list.
//named!(import_list,
//  ws!(do_parse!(
//    tag!("from") >>
//    module:
//  ))
//);

// Turn a &[u8] into a String.
fn bytes_to_string(bytes: &[u8]) -> String {
  unsafe { from_utf8_unchecked(bytes).to_owned() }
}

#[test]
fn test_module_sep_n_name() {
  assert_eq!(module_sep_n_name(&b".foo"[..]), IResult::Done(&b""[..], &b"foo"[..]));
  assert_eq!(module_sep_n_name(&b".foo.bar"[..]), IResult::Done(&b".bar"[..], &b"foo"[..]));
  assert_eq!(module_sep_n_name(&b"foo"[..]), IResult::Error(ErrorKind::Char));
  assert_eq!(module_sep_n_name(&b" .foo"[..]), IResult::Error(ErrorKind::Char));
  assert_eq!(module_sep_n_name(&b"."[..]), IResult::Incomplete(Needed::Unknown));
}

#[test]
fn test_module_path_simple() {
  assert_eq!(module_path(&b"foo"[..]), IResult::Done(&b""[..], syntax::ModulePath { path: vec!["foo".into()] }));
  assert_eq!(module_path(&b"foo "[..]), IResult::Done(&b" "[..], syntax::ModulePath { path: vec!["foo".into()] }));
  assert_eq!(module_path(&b"foo."[..]), IResult::Incomplete(Needed::Unknown));
  assert_eq!(module_path(&b" foo"[..]), IResult::Error(ErrorKind::AlphaNumeric));
}

#[test]
fn test_module_path_several() {
  assert_eq!(module_path(&b"foo.bar.zoo"[..]), IResult::Done(&b""[..], syntax::ModulePath { path: vec!["foo".into(), "bar".into(), "zoo".into()] }));
}

#[test]
fn test_module_list() {
  let foo = syntax::ModulePath { path: vec!["foo".into()] };
  let bar = syntax::ModulePath { path: vec!["bar".into()] };
  let zoo_woo = syntax::ModulePath { path: vec!["zoo".into(), "woo".into()] };
  let list = vec![foo, bar, zoo_woo];

  assert_eq!(module_list(&b"(foo,bar,zoo.woo)"[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(module_list(&b" ( foo,bar,zoo.woo  ) "[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(module_list(&b"( foo, bar ,   zoo.woo  )"[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(module_list(&b"(,bar,zoo.woo)"[..]), IResult::Error(ErrorKind::Char));
  assert_eq!(module_list(&b"("[..]), IResult::Incomplete(Needed::Unknown));
  assert_eq!(module_list(&b"  ("[..]), IResult::Incomplete(Needed::Unknown));
}

#[test]
fn test_export_list() {
  let foo = syntax::ModulePath { path: vec!["foo".into()] };
  let bar = syntax::ModulePath { path: vec!["bar".into()] };
  let zoo_woo = syntax::ModulePath { path: vec!["zoo".into(), "woo".into()] };
  let list = syntax::ExportList { export_list: vec![foo, bar, zoo_woo] };

  assert_eq!(export_list(&b"export (foo,bar,zoo.woo)"[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(export_list(&b"   export ( foo,bar,zoo.woo  )  "[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(export_list(&b"export ( foo, bar ,   zoo.woo  )"[..]), IResult::Done(&b""[..], list.clone()));
  assert_eq!(export_list(&b"export (,bar,zoo.woo)"[..]), IResult::Error(ErrorKind::Char));
  assert_eq!(export_list(&b"export ("[..]), IResult::Incomplete(Needed::Unknown));
  assert_eq!(export_list(&b"export   ("[..]), IResult::Incomplete(Needed::Unknown));
}
