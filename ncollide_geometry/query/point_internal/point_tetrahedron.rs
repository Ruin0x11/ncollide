use num::Bounded;
use na::{self, Real};
use shape::Tetrahedron;
use query::{PointProjection, PointQuery, PointQueryWithLocation, TrianglePointLocation};
use math::{Isometry, Point};
use utils;

impl<P: Point, M: Isometry<P>> PointQuery<P, M> for Tetrahedron<P> {
    #[inline]
    fn project_point(&self, m: &M, pt: &P, solid: bool) -> PointProjection<P> {
        let (projection, _) = self.project_point_with_location(m, pt, solid);
        projection
    }
}

/// Logical description of the location of a point on a triangle.
#[derive(Copy, Clone, Debug)]
pub enum TetrahedronPointLocation<N: Real> {
    /// The point lies on a vertex.
    OnVertex(usize),
    /// The point lies on a vertex.
    OnEdge(usize, [N; 2]),
    /// The point lies on a triangular face interior.
    ///
    /// The first face is the triangle ABC.
    /// The second face is the triangle ABD.
    /// The third face is the triangle ACD.
    /// The fourth face is the triangle BDC.
    OnFace(usize, [N; 3]),
    /// The point lies inside of the tetrahedron.
    OnSolid,
}

impl<N: Real> TetrahedronPointLocation<N> {
    /// Returns `true` if both `self` and `other` correspond to points on the same feature of a tetrahedron.
    pub fn same_feature_as(&self, other: &TetrahedronPointLocation<N>) -> bool {
        match (*self, *other) {
            (TetrahedronPointLocation::OnVertex(i), TetrahedronPointLocation::OnVertex(j)) => {
                i == j
            }
            (TetrahedronPointLocation::OnEdge(i, _), TetrahedronPointLocation::OnEdge(j, _)) => {
                i == j
            }
            (TetrahedronPointLocation::OnFace(i, _), TetrahedronPointLocation::OnFace(j, _)) => {
                i == j
            }
            (TetrahedronPointLocation::OnSolid, TetrahedronPointLocation::OnSolid) => true,
            _ => false,
        }
    }
}

impl<P: Point> Tetrahedron<P> {
    // NOTE: the following is the brute-force version of point-projection on tetrahedron.
    #[inline]
    #[allow(dead_code)]
    fn project_point_with_location2<M: Isometry<P>>(
        &self,
        m: &M,
        pt: &P,
        solid: bool,
    ) -> (PointProjection<P>, TetrahedronPointLocation<P::Real>) {
        let _0: P::Real = na::zero();

        let mut projs = [None; 4];

        let ap = *pt - *self.a();
        let ab = *self.b() - *self.a();
        let ac = *self.c() - *self.a();
        let ad = *self.d() - *self.a();
        let n1 = utils::cross3(&ab, &ac);
        if na::dot(&n1, &ad) * na::dot(&n1, &ap) <= _0 {
            let abc = self.face(0);
            projs[0] = Some(abc.project_point_with_location(m, pt, solid));
        }

        let bp = *pt - *self.b();
        let n2 = utils::cross3(&ab, &ad);
        if na::dot(&n2, &ac) * na::dot(&n2, &ap) <= _0 {
            let abd = self.face(1);
            projs[1] = Some(abd.project_point_with_location(m, pt, solid));
        }

        let n3 = utils::cross3(&ac, &ad);
        if na::dot(&n3, &ab) * na::dot(&n3, &ap) <= _0 {
            let acd = self.face(2);
            projs[2] = Some(acd.project_point_with_location(m, pt, solid));
        }

        let bc = *self.c() - *self.b();
        let bd = *self.d() - *self.b();
        let n4 = utils::cross3(&bc, &bd);
        if -na::dot(&n4, &ab) * na::dot(&n4, &bp) <= _0 {
            let bcd = self.face(3);
            projs[3] = Some(bcd.project_point_with_location(m, pt, solid));
        }

        let mut smallest_id = 42;
        let mut smallest_dist = P::Real::max_value();
        for i in 0..4 {
            if let Some((ref proj, _)) = projs[i] {
                let dist = na::norm_squared(&(proj.point - *pt));
                if dist < smallest_dist {
                    smallest_id = i;
                    smallest_dist = dist;
                }
            }
        }

        if smallest_id == 42 {
            return (
                PointProjection::new(true, *pt),
                TetrahedronPointLocation::OnSolid,
            );
        }

        /*
         * 0 AB
         * 1 AC
         * 2 AD
         * 3 BC
         * 4 BD
         * 5 CD
         */
        let edges = [[0, 3, 1], [0, 4, 2], [1, 5, 2], [3, 5, 4]];

        let vertices = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

        if let Some((ref proj, ref loc)) = projs[smallest_id] {
            let new_loc = match *loc {
                TrianglePointLocation::OnVertex(i) => {
                    TetrahedronPointLocation::OnVertex(vertices[smallest_id][i])
                }
                TrianglePointLocation::OnEdge(i, uv) => {
                    TetrahedronPointLocation::OnEdge(edges[smallest_id][i], uv)
                }
                TrianglePointLocation::OnFace(uv) => {
                    TetrahedronPointLocation::OnFace(smallest_id, uv)
                }
                TrianglePointLocation::OnSolid => TetrahedronPointLocation::OnSolid,
            };

            (*proj, new_loc)
        } else {
            unreachable!()
        }
    }
}

impl<P: Point, M: Isometry<P>> PointQueryWithLocation<P, M> for Tetrahedron<P> {
    type Location = TetrahedronPointLocation<P::Real>;

    #[inline]
    fn project_point_with_location(
        &self,
        m: &M,
        pt: &P,
        solid: bool,
    ) -> (PointProjection<P>, Self::Location) {
        let p = m.inverse_transform_point(pt);

        let ab = *self.b() - *self.a();
        let ac = *self.c() - *self.a();
        let ad = *self.d() - *self.a();
        let ap = p - *self.a();

        /*
         * Voronoï regions of vertices.
         */
        let ap_ab = na::dot(&ap, &ab);
        let ap_ac = na::dot(&ap, &ac);
        let ap_ad = na::dot(&ap, &ad);

        let _0: P::Real = na::zero();

        if ap_ab <= _0 && ap_ac <= _0 && ap_ad <= _0 {
            // Voronoï region of `a`.
            let proj = PointProjection::new(false, m.transform_point(self.a()));
            return (proj, TetrahedronPointLocation::OnVertex(0));
        }

        let bc = *self.c() - *self.b();
        let bd = *self.d() - *self.b();
        let bp = p - *self.b();

        let bp_bc = na::dot(&bp, &bc);
        let bp_bd = na::dot(&bp, &bd);
        let bp_ab = na::dot(&bp, &ab);

        if bp_bc <= _0 && bp_bd <= _0 && bp_ab >= _0 {
            // Voronoï region of `b`.
            let proj = PointProjection::new(false, m.transform_point(self.b()));
            return (proj, TetrahedronPointLocation::OnVertex(1));
        }

        let cd = *self.d() - *self.c();
        let cp = p - *self.c();

        let cp_ac = na::dot(&cp, &ac);
        let cp_bc = na::dot(&cp, &bc);
        let cp_cd = na::dot(&cp, &cd);

        if cp_cd <= _0 && cp_bc >= _0 && cp_ac >= _0 {
            // Voronoï region of `c`.
            let proj = PointProjection::new(false, m.transform_point(self.c()));
            return (proj, TetrahedronPointLocation::OnVertex(2));
        }

        let dp = p - *self.d();

        let dp_cd = na::dot(&dp, &cd);
        let dp_bd = na::dot(&dp, &bd);
        let dp_ad = na::dot(&dp, &ad);

        if dp_ad >= _0 && dp_bd >= _0 && dp_cd >= _0 {
            // Voronoï region of `d`.
            let proj = PointProjection::new(false, m.transform_point(self.d()));
            return (proj, TetrahedronPointLocation::OnVertex(3));
        }

        /*
         * Voronoï regions of edges.
         */
        #[inline(always)]
        fn check_edge<P, M>(
            i: usize,
            m: &M,
            a: &P,
            b: &P,
            nabc: &P::Vector,
            nabd: &P::Vector,
            ap: &P::Vector,
            ab: &P::Vector,
            ap_ab: P::Real, /*ap_ac: P::Real, ap_ad: P::Real,*/
            bp_ab: P::Real, /*bp_ac: P::Real, bp_ad: P::Real*/
        ) -> (
            P::Real,
            P::Real,
            Option<(PointProjection<P>, TetrahedronPointLocation<P::Real>)>,
        )
        where
            P: Point,
            M: Isometry<P>,
        {
            let _0: P::Real = na::zero();
            let _1: P::Real = na::one();

            let ab_ab = ap_ab - bp_ab;

            // NOTE: The following avoids the subsequent cross and dot products but are not
            // numerically stable.
            //
            // let dabc  = ap_ab * (ap_ac - bp_ac) - ap_ac * ab_ab;
            // let dabd  = ap_ab * (ap_ad - bp_ad) - ap_ad * ab_ab;

            let ap_x_ab = utils::cross3(ap, ab);
            let dabc = na::dot(&ap_x_ab, nabc);
            let dabd = na::dot(&ap_x_ab, nabd);

            if dabc >= _0 && dabd >= _0 && ap_ab >= _0 && ap_ab <= ab_ab {
                // Voronoi region of `ab`.
                assert!(ab_ab != _0);
                let u = ap_ab / ab_ab;
                let bcoords = [_1 - u, u];
                let mut res = *a;
                res.axpy(bcoords[1], b, bcoords[0]);
                let proj = PointProjection::new(false, m.transform_point(&res));
                (
                    dabc,
                    dabd,
                    Some((proj, TetrahedronPointLocation::OnEdge(i, bcoords))),
                )
            } else {
                (dabc, dabd, None)
            }
        }

        // Voronoï region of ab.
        //            let bp_ad = bp_bd + bp_ab;
        //            let bp_ac = bp_bc + bp_ab;
        let nabc = utils::cross3(&ab, &ac);
        let nabd = utils::cross3(&ab, &ad);
        let (dabc, dabd, res) = check_edge(
            0,
            m,
            self.a(),
            self.b(),
            &nabc,
            &nabd,
            &ap,
            &ab,
            ap_ab,
            /*ap_ac, ap_ad,*/ bp_ab, /*, bp_ac, bp_ad*/
        );
        if let Some(res) = res {
            return res;
        }

        // Voronoï region of ac.
        // Substitutions (wrt. ab):
        //   b -> c
        //   c -> d
        //   d -> b
        //            let cp_ab = cp_ac - cp_bc;
        //            let cp_ad = cp_cd + cp_ac;
        let nacd = utils::cross3(&ac, &ad);
        let (dacd, dacb, res) = check_edge(
            1,
            m,
            self.a(),
            self.c(),
            &nacd,
            &-nabc,
            &ap,
            &ac,
            ap_ac,
            /*ap_ad, ap_ab,*/ cp_ac, /*, cp_ad, cp_ab*/
        );
        if let Some(res) = res {
            return res;
        }

        // Voronoï region of ad.
        // Substitutions (wrt. ab):
        //   b -> d
        //   c -> b
        //   d -> c
        //            let dp_ac = dp_ad - dp_cd;
        //            let dp_ab = dp_ad - dp_bd;
        let (dadb, dadc, res) = check_edge(
            2,
            m,
            self.a(),
            self.d(),
            &-nabd,
            &-nacd,
            &ap,
            &ad,
            ap_ad,
            /*ap_ab, ap_ac,*/ dp_ad, /*, dp_ab, dp_ac*/
        );
        if let Some(res) = res {
            return res;
        }

        // Voronoï region of bc.
        // Substitutions (wrt. ab):
        //   a -> b
        //   b -> c
        //   c -> a
        //            let cp_bd = cp_cd + cp_bc;
        let nbcd = utils::cross3(&bc, &bd);
        // NOTE: nabc = nbcd
        let (dbca, dbcd, res) = check_edge(
            3,
            m,
            self.b(),
            self.c(),
            &nabc,
            &nbcd,
            &bp,
            &bc,
            bp_bc,
            /*-bp_ab, bp_bd,*/ cp_bc, /*, -cp_ab, cp_bd*/
        );
        if let Some(res) = res {
            return res;
        }

        // Voronoï region of bd.
        // Substitutions (wrt. ab):
        //   a -> b
        //   b -> d
        //   d -> a

        //            let dp_bc = dp_bd - dp_cd;
        // NOTE: nbdc = -nbcd
        // NOTE: nbda = nabd
        let (dbdc, dbda, res) = check_edge(
            4,
            m,
            self.b(),
            self.d(),
            &-nbcd,
            &nabd,
            &bp,
            &bd,
            bp_bd,
            /*bp_bc, -bp_ab,*/ dp_bd, /*, dp_bc, -dp_ab*/
        );
        if let Some(res) = res {
            return res;
        }

        // Voronoï region of cd.
        // Substitutions (wrt. ab):
        //   a -> c
        //   b -> d
        //   c -> a
        //   d -> b
        // NOTE: ncda = nacd
        // NOTE: ncdb = nbcd
        let (dcda, dcdb, res) = check_edge(
            5,
            m,
            self.c(),
            self.d(),
            &nacd,
            &nbcd,
            &cp,
            &cd,
            cp_cd,
            /*-cp_ac, -cp_bc,*/ dp_cd, /*, -dp_ac, -dp_bc*/
        );
        if let Some(res) = res {
            return res;
        }

        /*
         * Voronoï regions of faces.
         */
        #[inline(always)]
        fn check_face<P, M>(
            i: usize,
            a: &P,
            b: &P,
            c: &P,
            m: &M,
            ap: &P::Vector,
            bp: &P::Vector,
            cp: &P::Vector,
            ab: &P::Vector,
            ac: &P::Vector,
            ad: &P::Vector,
            dabc: P::Real,
            dbca: P::Real,
            dacb: P::Real,
            /* ap_ab: P::Real, bp_ab: P::Real, cp_ab: P::Real,
                                   ap_ac: P::Real, bp_ac: P::Real, cp_ac: P::Real, */
        ) -> Option<(PointProjection<P>, TetrahedronPointLocation<P::Real>)>
        where
            P: Point,
            M: Isometry<P>,
        {
            let _0: P::Real = na::zero();
            let _1: P::Real = na::one();

            if dabc < _0 && dbca < _0 && dacb < _0 {
                let n = utils::cross3(ab, ac); // FIXME: is is possible to avoid this cross product?
                if na::dot(&n, ad) * na::dot(&n, ap) < _0 {
                    // Voronoï region of the face.

                    // NOTE:
                    // The following avoids expansive computations but are not very
                    // numerically stable.
                    //
                    // let va = bp_ab * cp_ac - cp_ab * bp_ac;
                    // let vb = cp_ab * ap_ac - ap_ab * cp_ac;
                    // let vc = ap_ab * bp_ac - bp_ab * ap_ac;

                    let normal = na::normalize(&n);
                    let vc = na::dot(&normal, &utils::cross3(&ap, &bp));
                    let va = na::dot(&normal, &utils::cross3(&bp, &cp));
                    let vb = na::dot(&normal, &utils::cross3(&cp, &ap));

                    let denom = va + vb + vc;
                    assert!(denom != _0);
                    let inv_denom = _1 / denom;

                    let bcoords = [va * inv_denom, vb * inv_denom, vc * inv_denom];
                    let mut res = *a;
                    res.axpy(bcoords[1], b, bcoords[0]);
                    res.axpy(bcoords[2], c, _1);
                    let proj = PointProjection::new(false, m.transform_point(&res));

                    return Some((proj, TetrahedronPointLocation::OnFace(i, bcoords)));
                }
            }
            return None;
        }

        // Face abc.
        if let Some(res) = check_face(
            0,
            self.a(),
            self.b(),
            self.c(),
            m,
            &ap,
            &bp,
            &cp,
            &ab,
            &ac,
            &ad,
            dabc,
            dbca,
            dacb,
            /*ap_ab, bp_ab, cp_ab,
                                          ap_ac, bp_ac, cp_ac*/
        ) {
            return res;
        }

        // Face abd.
        if let Some(res) = check_face(
            1,
            self.a(),
            self.b(),
            self.d(),
            m,
            &ap,
            &bp,
            &dp,
            &ab,
            &ad,
            &ac,
            dadb,
            dabd,
            dbda,
            /*ap_ab, bp_ab, dp_ab,
                                          ap_ad, bp_ad, dp_ad*/
        ) {
            return res;
        }
        // Face acd.
        if let Some(res) = check_face(
            2,
            self.a(),
            self.c(),
            self.d(),
            m,
            &ap,
            &cp,
            &dp,
            &ac,
            &ad,
            &ab,
            dacd,
            dcda,
            dadc,
            /*ap_ac, cp_ac, dp_ac,
                                          ap_ad, cp_ad, dp_ad*/
        ) {
            return res;
        }
        // Face bcd.
        if let Some(res) = check_face(
            3,
            self.b(),
            self.c(),
            self.d(),
            m,
            &bp,
            &cp,
            &dp,
            &bc,
            &bd,
            &-ab,
            dbcd,
            dcdb,
            dbdc,
            /*bp_bc, cp_bc, dp_bc,
                                          bp_bd, cp_bd, dp_bd*/
        ) {
            return res;
        }

        if !solid {
            unimplemented!("Non-solid ray-cast on a tetrahedron is not yet implemented.")
        }

        let proj = PointProjection::new(true, m.transform_point(&p));
        return (proj, TetrahedronPointLocation::OnSolid);
    }
}
