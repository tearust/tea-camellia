#![cfg_attr(not(feature = "std"), no_std)]

use bounding_curve_interface::BoundingCurveInterface;
use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};
use sp_std::{convert::TryInto, marker::PhantomData};

const CENTS: node_primitives::Balance = 10_000_000_000;
const DOLLARS: node_primitives::Balance = 100 * CENTS;

pub mod linear;
pub mod square_root;
