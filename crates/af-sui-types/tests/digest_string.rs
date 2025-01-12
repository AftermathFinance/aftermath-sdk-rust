use af_sui_types::Digest;

const FROM_GQL: &[&str] = &[
    "BZHvCMQA756jME1t5DbbUzVQX32PNQsP1H3KftpNeTy4",
    "3fcDagxR4vYWqHym8pR8wqLwvg26BuVenxdfigsdArVA",
];

#[test]
fn display() {
    for &string in FROM_GQL {
        let digest: Digest = string.parse().unwrap();
        assert_eq!(format!("{digest}"), string);
        assert_eq!(format!("{digest:#}"), string);
    }
}

#[test]
fn debug() {
    for &string in FROM_GQL {
        let digest: Digest = string.parse().unwrap();
        assert_eq!(format!("{digest:?}"), format!("Digest(\"{string}\")"));
        assert_eq!(
            format!("{digest:#?}"),
            format!("Digest(\n    \"{string}\",\n)")
        );
    }
}
