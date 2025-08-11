pub mod finbert_model {
    include!(concat!(env!("OUT_DIR"), "/model/finbert_model.rs"));
}

pub use finbert_model::Model as FinBertModel;
