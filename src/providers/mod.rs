pub mod apibay;
pub mod eztv;
pub mod knaben;
pub mod nyaa;
pub mod yts;

use crate::provider::Provider;

pub fn all_providers() -> Vec<(&'static str, Box<dyn Provider>)> {
    vec![
        ("apibay", Box::new(apibay::Apibay)),
        ("yts", Box::new(yts::Yts)),
        ("knaben", Box::new(knaben::Knaben)),
        ("nyaa", Box::new(nyaa::Nyaa)),
        ("eztv", Box::new(eztv::Eztv)),
    ]
}
