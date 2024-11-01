use futures::{StreamExt, TryStreamExt};
use jsonrpsee_core::client::{ClientT, SubscriptionClientT, SubscriptionKind};
use jsonrpsee_ws_client::types::SubscriptionId;
use std::sync::Arc;
use subxt::{backend::rpc::*, OnlineClient, PolkadotConfig};

type BoxError = Box<dyn std::error::Error>;

#[derive(Clone)]
struct RpcClient(Arc<jsonrpsee_ws_client::WsClient>);

impl RpcClient {
    pub async fn new(url: &str) -> Result<Self, BoxError> {
        let client = jsonrpsee_ws_client::WsClientBuilder::default()
            .build(url)
            .await
            .map_err(|e| Box::new(e))?;
        Ok(RpcClient(Arc::new(client)))
    }
}

struct RpcParams(Option<Box<RawValue>>);

impl jsonrpsee_core::traits::ToRpcParams for RpcParams {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        Ok(self.0)
    }
}

impl RpcClientT for RpcClient {
    fn request_raw<'a>(
        &'a self,
        method: &'a str,
        params: Option<Box<RawValue>>,
    ) -> RawRpcFuture<'a, Box<RawValue>> {
        Box::pin(async move {
            self.0
                .request(method, RpcParams(params))
                .await
                .map_err(|e| subxt::error::RpcError::ClientError(Box::new(e)))
        })
    }

    fn subscribe_raw<'a>(
        &'a self,
        sub: &'a str,
        params: Option<Box<RawValue>>,
        unsub: &'a str,
    ) -> RawRpcFuture<'a, RawRpcSubscription> {
        Box::pin(async move {
            let sub = self
                .0
                .subscribe(sub, RpcParams(params), unsub)
                .await
                .map_err(|e| subxt::error::RpcError::ClientError(Box::new(e)))?;

            let id = match &sub.kind() {
                &SubscriptionKind::Subscription(SubscriptionId::Num(n)) => n.to_string(),
                &SubscriptionKind::Subscription(SubscriptionId::Str(s)) => s.to_string(),
                _ => panic!("unexpected method subscription"),
            };

            let stream = sub
                .map_err(|e| subxt::error::RpcError::ClientError(Box::new(e)))
                .boxed();

            Ok(RawRpcSubscription {
                stream,
                id: Some(id),
            })
        })
    }
}

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
pub mod runtime {}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let client = RpcClient::new("ws://127.0.0.1:9944").await?;
    let subxt: OnlineClient<PolkadotConfig> = OnlineClient::from_rpc_client(client).await?;

    let client = jsonrpsee_ws_client::WsClientBuilder::default()
        .build("ws://127.0.0.1:9944")
        .await?;

    Ok(())
}
