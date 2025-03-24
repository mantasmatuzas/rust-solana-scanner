use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    let request_login = hc.do_post(
        "/api/login",
        json!({
            "username": "mantas",
            "password": "welcome",
        }),
    );
    request_login.await?.print().await?;

    let req_create_ticket = hc.do_post(
        "/api/tickets",
        json!({
            "title": "Ticket Kautra"
        }),
    );
    req_create_ticket.await?.print().await?;

    hc.do_delete("/api/tickets/1").await?.print().await?;

    hc.do_get("/api/tickets").await?.print().await?;

    hc.do_get("/api/solana/block/354587721").await?.print().await?;

    Ok(())
}
