pub mod private;
pub mod public;
pub mod signature;

#[cfg(test)]
mod tests {
    use crate::ed25519::{private::PrivateKey, signature::Signature};

    #[test]
    fn test_sign_and_verify() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let message = b"test message";

        let signature = private_key.sign(message);
        assert!(public_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_verify_with_invalid_signature() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let message = b"test message";
        let invalid_signature = Signature::from([0u8; 64]);

        // Verify the invalid signature
        assert!(public_key.verify(message, &invalid_signature).is_err());
    }

    #[test]
    fn test_verify_with_different_message() {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let message = b"test message";
        let different_message = b"different message";

        // Sign the message
        let signature = private_key.sign(message);

        // Verify the signature with a different message
        assert!(public_key.verify(different_message, &signature).is_err());
    }
}
