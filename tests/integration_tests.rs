#[cfg(test)]
mod server_tests {
    use std::time::Duration;
    use std::{thread, time};
    use async_std::{task};

    #[async_std::test]
    async fn test_udp() {

        task::spawn(async {
            rperf::start_server(7777, "udp", false).await.unwrap();
        });

        let result= rperf::start_test("127.0.0.1", 7777, "udp", Duration::from_secs(1), 1000, 64, "", true, true).await;
        assert!(result.is_ok())
    }

    #[async_std::test]
    async fn test_tcp() {

        task::spawn(async {
            rperf::start_server(7778, "tcp", false).await.unwrap();
        });

        thread::sleep(time::Duration::from_millis(100));

        let result= rperf::start_test("127.0.0.1", 7778, "tcp", Duration::from_secs(1), 1000, 64, "", true, true).await;
        assert!(result.is_ok())
    }

    #[async_std::test]
    async fn unsupported_protocol() {
        let exception_thrown = async_std::io::timeout(Duration::from_secs(1), async {
            match rperf::start_server(5555, "ABC", false).await {
                Ok(_) => {
                    Ok(false)
                }
                Err(_) => {
                    Ok(true)
                }
            }
        }).await.unwrap();

        assert!(exception_thrown)
    }
}
