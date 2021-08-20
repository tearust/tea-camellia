#![cfg_attr(not(feature = "std"), no_std)]

use bounding_curve_interface::BoundingCurve;
use sp_runtime::traits::{AtLeast32BitUnsigned, One};

pub mod linear;
pub mod square_root;
