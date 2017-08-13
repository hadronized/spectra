use linear::V3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
  pub lower: V3<f32>,
  pub upper: V3<f32>
}

impl AABB {
  /// Construct an AABB from vertices. Return `None` if there’s no vertex.
  pub fn from_vertices<I>(mut vertices: I) -> Option<Self> where I: Iterator<Item = V3<f32>> {
    vertices.next().map(move |first| {
      let (lower, upper) = vertices.fold((first, first), |acc, xyz| {
        let min = V3::new(acc.0.x.min(xyz.x), acc.0.y.min(xyz.y), acc.0.z.min(xyz.z));
        let max = V3::new(acc.1.x.max(xyz.x), acc.1.y.max(xyz.y), acc.1.z.max(xyz.z));
        (min, max)
      });

      AABB { lower, upper }
    })
  }

  /// Construct an AABB from AABBs. Return `None` if there’s no AABB.
  pub fn from_aabbs<I>(mut aabbs: I) -> Option<Self> where I: Iterator<Item = AABB>{
    aabbs.next().map(move |first| {
      aabbs.fold(first, |acc, bb| {
        let vertices = [acc.lower, acc.upper, bb.lower, bb.upper];
        Self::from_vertices(vertices.into_iter().cloned()).unwrap()
      })
    })
  }
}
