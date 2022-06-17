use pluto_network::key::*;

/// Helper for key generation in coordinator initial setup.
pub fn generate_print_initial_keys() {
    let keypair = Keys::generate();

    println!("Generated a new keypair for coordinator:\n");
    println!("COORDINATOR_PUBKEY=\"{}\"", base64::encode(keypair.public_key().as_bytes()));
    println!("COORDINATOR_PRIVKEY=\"{}\"", base64::encode(keypair.private_key().to_bytes()));
}
