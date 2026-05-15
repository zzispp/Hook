use serde::Serialize;

pub fn build_path_with_query<T: Serialize>(path: &str, query: &T) -> Result<String, serde_urlencoded::ser::Error> {
    let query_string = serde_urlencoded::to_string(query)?;
    Ok(format!("{path}?{query_string}"))
}

#[cfg(test)]
mod tests {
    use super::build_path_with_query;
    use serde::Serialize;

    #[derive(Serialize)]
    struct CoinQuery {
        pub market_data: bool,
        pub community_data: bool,
        pub tickers: bool,
        pub localization: bool,
        pub developer_data: bool,
    }

    #[test]
    fn builds_expected_query_string() {
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let result = build_path_with_query("/api/v3/coins/bitcoin", &query).unwrap();
        assert_eq!(
            result,
            "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true"
        );
    }
}
