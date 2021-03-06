//! Miscelaneous elementary geometric utilities.

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(missing_docs)]
#![deny(unused_results)]
#![warn(unused_imports)]
#![allow(missing_copy_implementations)] // FIXME: deny this.
#![doc(html_root_url = "http://ncollide.org/rustdoc")]

extern crate alga;
#[macro_use]
extern crate approx;
extern crate nalgebra as na;
extern crate ncollide_math as math;
extern crate num_traits as num;
extern crate rand;

pub use center::center;
// pub use project_homogeneous::{project_homogeneous, project_homogeneous_to};
pub use triangle::{circumcircle, is_point_in_triangle, triangle_area, triangle_center,
                   triangle_perimeter, is_affinely_dependent_triangle3};
pub use tetrahedron::{tetrahedron_center, tetrahedron_signed_volume, tetrahedron_volume};
pub use cleanup::remove_unused_points;
pub use derivatives::{binom, dcos, dsin};
// pub use optimization::{maximize_with_newton, newton, minimize_with_bfgs, bfgs,
//                        LineSearch, BacktrackingLineSearch};
pub use hashable_partial_eq::HashablePartialEq;
#[doc(inline)]
pub use as_bytes::AsBytes;
// pub use cov::{cov, cov_and_center, center_reduce};
pub use median::median;
pub use sort::sort3;
pub use cross3::cross3;
pub use perp2::perp2;
pub use point_cloud_support_point::point_cloud_support_point;
pub use repeat::repeat;

pub mod data;
mod center;
// mod project_homogeneous;
mod tetrahedron;
mod triangle;
mod cleanup;
mod derivatives;
// mod optimization;
mod hashable_partial_eq;
#[doc(hidden)]
pub mod as_bytes;
// mod cov;
mod median;
mod sort;
mod cross3;
mod perp2;
mod point_cloud_support_point;
mod repeat;
