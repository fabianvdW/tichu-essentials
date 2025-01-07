#![allow(unused_imports)]
pub mod tichu_hand;
pub mod enumerate_hands;
pub mod countable_properties;
pub mod enumeration_results;
pub mod bsw_database;
pub mod street_detection_tricks;
pub mod pair_street_detection_trick;
pub mod bsw_binary_format;
pub mod analysis;

use pyo3::prelude::*;
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + 2 * b).to_string())
}


#[pyclass]
pub struct BSWSimple {
    #[pyo3(get, set)]
    pub rounds: Vec<[u8; 4]>,
}
#[pymethods]
impl BSWSimple {
    #[new]
    pub fn new() -> BSWSimple {
        BSWSimple { rounds: vec![] }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tichu_rustipy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BSWSimple>()?;
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}