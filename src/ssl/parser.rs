use nom::{ErrorKind, IResult, Needed, alphanumeric};

use ssl::syntax;

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
named!(module_path<&[u8], Vec<&[u8]>>,
  do_parse!(
    // recognize at least one module name
    base: alphanumeric >>
    // recognize the rest of the path, if any
    rest: many0!(module_sep_n_name) >>

    ({
      let mut rest = rest.clone(); // FIXME: meh?
      rest.insert(0, base);
      rest
    })
  )
);
// 
// /// Parse a module list.
// ///
// ///     ( item0, item1, item2, …)
// named!(module_list, ws!(delimited!(char!('('), separated_list!(module_path, char!(',')), char!(')'))));
// 
// /// Parse an export list.
// named!(export_list<&[u8], ExportList>,
//   ws!(do_parse!(
//     tag!("export") >>
//     modules: module_list >>
//     (ExportList {
//       exported_list: modules.into_iter().collect()
//     })
//   ))
// );

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
  assert_eq!(module_path(&b"foo"[..]), IResult::Done(&b""[..], vec![&b"foo"[..]]));
  assert_eq!(module_path(&b"foo "[..]), IResult::Done(&b" "[..], vec![&b"foo"[..]]));
  assert_eq!(module_path(&b"foo."[..]), IResult::Incomplete(Needed::Unknown));
  assert_eq!(module_path(&b" foo"[..]), IResult::Error(ErrorKind::AlphaNumeric));
}

#[test]
fn test_module_path_several() {
  assert_eq!(module_path(&b"foo.bar.zoo"[..]), IResult::Done(&b""[..], vec![&b"foo"[..], &b"bar"[..], &b"zoo"[..]]));
}
