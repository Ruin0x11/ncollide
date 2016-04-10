use na::{Identity, Translate};
use na;
use math::{Point, Vect, Isometry};
use entities::bounding_volume::{self, HasBoundingVolume, AABB};
use entities::partitioning::BVTCostFn;
use entities::inspection::Repr;
use entities::shape::CompositeShape;
use ray::{Ray, RayCast};
use geometry::time_of_impact_internal;

/// Time Of Impact of a composite shape with any other shape, under translational movement.
pub fn composite_shape_against_any<P, M, G1: ?Sized, G2: ?Sized>(m1: &M, vel1: &P::Vect, g1: &G1,
                                                                 m2: &M, vel2: &P::Vect, g2: &G2)
                                                                 -> Option<<P::Vect as Vect>::Scalar>
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Repr<P, M> + HasBoundingVolume<M, AABB<P>> {
    let mut cost_fn = CompositeShapeAgainstAnyTOICostFn::new(m1, vel1, g1, m2, vel2, g2);

    g1.bvt().best_first_search(&mut cost_fn).map(|(_, res)| res)
}

/// Time Of Impact of any shape with a composite shape, under translational movement.
pub fn any_against_composite_shape<P, M, G1: ?Sized, G2: ?Sized>(m1: &M, vel1: &P::Vect, g1: &G1,
                                                                 m2: &M, vel2: &P::Vect, g2: &G2)
                                                                 -> Option<<P::Vect as Vect>::Scalar>
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect>,
          G1: Repr<P, M> + HasBoundingVolume<M, AABB<P>>,
          G2: CompositeShape<P, M> {
    composite_shape_against_any(m2, vel2, g2, m1, vel1, g1)
}

struct CompositeShapeAgainstAnyTOICostFn<'a, P: 'a + Point, M: 'a, G1: ?Sized + 'a, G2: ?Sized + 'a> {
    msum_shift:  P::Vect,
    msum_margin: P::Vect,
    ray:         Ray<P>,

    m1:   &'a M,
    vel1: &'a P::Vect,
    g1:   &'a G1,
    m2:   &'a M,
    vel2: &'a P::Vect,
    g2:   &'a G2
}

impl<'a, P, M, G1: ?Sized, G2: ?Sized> CompositeShapeAgainstAnyTOICostFn<'a, P, M, G1, G2>
    where P:  Point,
          M:  Isometry<P, P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Repr<P, M> + HasBoundingVolume<M, AABB<P>> {
    pub fn new(m1: &'a M, vel1: &'a P::Vect, g1: &'a G1, m2: &'a M, vel2: &'a P::Vect, g2: &'a G2)
        -> CompositeShapeAgainstAnyTOICostFn<'a, P, M, G1, G2> {

        let ls_m2 = na::inv(m1).expect("The transformation `m1` must be inversible.") * *m2;
        let ls_aabb2 = bounding_volume::aabb(g2, &ls_m2);

        CompositeShapeAgainstAnyTOICostFn {
            msum_shift:  -ls_aabb2.center().to_vec(),
            msum_margin: ls_aabb2.half_extents(),
            ray:         Ray::new(na::orig(), m1.inv_rotate(&(*vel2 - *vel1))),
            m1:          m1,
            vel1:        vel1,
            g1:          g1,
            m2:          m2,
            vel2:        vel2,
            g2:          g2
        }
    }
}

impl<'a, P, M, G1: ?Sized, G2: ?Sized> BVTCostFn<<P::Vect as Vect>::Scalar, usize, AABB<P>>
for CompositeShapeAgainstAnyTOICostFn<'a, P, M, G1, G2>
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Repr<P, M> + HasBoundingVolume<M, AABB<P>> {
    type UserData = <P::Vect as Vect>::Scalar;

    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<P>) -> Option<<P::Vect as Vect>::Scalar> {
        // Compute the minkowski sum of the two AABBs.
        let msum = AABB::new(*bv.mins() + self.msum_shift + (-self.msum_margin),
                             *bv.maxs() + self.msum_shift + self.msum_margin);

        // Compute the TOI.
        msum.toi_with_ray(&Identity::new(), &self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(<P::Vect as Vect>::Scalar, <P::Vect as Vect>::Scalar)> {
        let mut res = None;

        self.g1.map_transformed_part_at(self.m1, *b, &mut |m1, g1|
            res = time_of_impact_internal::time_of_impact(m1, self.vel1, g1, self.m2, self.vel2, self.g2)
                  .map(|toi| (toi, toi))
        );

        res
    }
}
