#![cfg_attr(not(feature = "std"), no_std)]

use bounding_curve_interface::{BuyBoundingCurve, SellBoundingCurve};
use sp_runtime::traits::AtLeast32BitUnsigned;

pub mod linear;
