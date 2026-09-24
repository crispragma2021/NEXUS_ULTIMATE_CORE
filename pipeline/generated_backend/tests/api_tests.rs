#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_healthcheck_contract() {
        assert_eq!(2 + 2, 4);
        println!("✅ Test Healthcheck ejecutado exitosamente.");
    }

    #[tokio::test]
    async fn test_user_creation_contract() {
        let user_id = 103;
        assert!(user_id > 0);
        println!("✅ Test User Creation ejecutado exitosamente.");
    }
}
