# 0.7.0

- Expose a framerate limit in CLI.
- Update all the dependencies.
- Add Gitter badge.
- Enhanced a little bit the documentation (yet still very bad).
- Some internal fixed about `bootstrap::Device::events`.
- All the hot-reloading resource code now lives in the [warmy](https://crates.io/crates/warmy)
  crate.
- The Cheddar shading language is now the default – and only – way to build shaders.
- Internal refactoring.

# 0.6

- New resource system, with types keys, lasers, ninja and shit.

# 0.5

- Support for luminance-0.22.
- Changed the way models are handled (`ModelTree` and `MaterialTree`).

## 0.4.3

## 0.4.2

- Support for serde{,-*}-1.0.
- Support for image-0.13.

## 0.4.1

- Support for luminance-glfw-0.3.

# 0.4

- Implemented canonicalized resource cache. Such a cache can now fail when you create it if the root
  path doesn’t exist. All resources are internally stored in a canonicalized way so that no
  ambiguities can occur.

# 0.3

- `ResCache::get_proxied` now returns `Result<_>` as well if the path points to something that
  doesn’t exist (hot-reloading won’t work for that).

## 0.2.6

- Hot reloading fix (especially, it works on Mac OSX now).

## 0.2.5

- luminance-0.2.5 fix.

## 0.2.4

- Added `From / Into` impls for `[[f32; 4]; 4]` for `Transform`.

## 0.2.3

- `ResCache::get_proxied` doesn’t return an `Option<T>` anymore but a `T`, as it should.

## 0.2.2

- Made all `TextureImage`’s fields pub.

## 0.2.1

- Added several constructors for `Program`. Among them, `from_bufread` and `from_str`.
- Added `ResCache::get_proxied`, which takes an extra argument (regarding `ResCache::get`), a
  closure, used to compute a proxy value that will be used if resource loading fails.

# 0.2

- Added a `bootstrap!` macro used to build a `Device` in a very simple way (relying on clap).
- Fixed docs.rs link.
- luminance-glfw integration.
- Removed texture handling from the compositor and simplified it a lot.
- Switched to cgmath and dropped nalgebra.
- Refactored all spectra code according to typed programs (luminance).

# 0.1

- Initial revision.
