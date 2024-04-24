use starknet_api::hash::{pedersen_hash, StarkFelt};

/// Computes the Pedersen root hash of the given data.
pub fn pedersen_root(data: Vec<StarkFelt>) -> StarkFelt {
    if data.is_empty() {
        return StarkFelt::ZERO;
    }
    let mut layer = data;
    while layer.len() > 1 {
        layer = layer
            .chunks(2)
            .map(|chunk| match chunk {
                [a] => pedersen_hash(a, a),
                // [a] => pedersen_hash(a, StarkFelt::ZERO),
                [a, b] => pedersen_hash(a, b),
                _ => unreachable!(),
            })
            .collect();
    }
    layer[0]
}
