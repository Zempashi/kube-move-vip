use futures::{StreamExt, TryStreamExt, stream};
use k8s_openapi::api::{
    core::v1::{Node},
};
use k8s_openapi::api::coordination::v1::Lease;
use kube::{
    Client,
    api::{Api, ResourceExt},
    runtime::{WatchStreamExt, watcher},
};
use tracing::*;
use rustls;
use std::collections::HashMap;

mod settings;
mod oneprovider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let settings = settings::Config::new().unwrap();
    let oneprovider = oneprovider::Oneprovider::new(settings.provider.oneprovider);
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;


    //let cilium = cilium::CiliumSvc::new();
    //cilium.cilium_controller(client).await;
    let lease = Api::<Lease>::namespaced(client.clone(), "kube-system");
    let nodes = Api::<Node>::all(client.clone());
    //let service = Api::<Service>::all(client.clone());
    let lease_watcher = watcher(lease, watcher::Config::default());
    let node_watcher = watcher(nodes, watcher::Config::default());

    // select on applied events from all watchers
    let mut combo_stream = stream::select_all(vec![
        lease_watcher.applied_objects().map_ok(Watched::Lease).boxed(),
        node_watcher.applied_objects().map_ok(Watched::Node).boxed(),
    ]);
    // SelectAll Stream elements must have the same Item, so all packed in this:
    #[allow(clippy::large_enum_variant)]
    enum Watched {
        Lease(Lease),
        Node(Node),
    }
    let mut failover_position = HashMap::<String, String>::new();
    let failover_ip = "51.158.30.238";
    info!("Controller starting for {}...", failover_ip);
    while let Some(o) = combo_stream.try_next().await? {
        match o {
            Watched::Lease(lease) => {
                debug!("Got Lease: {}", lease.name_any());
                if lease.name_any() == "cilium-l2announce-traefik-traefik" {
                    debug!("Got matching lease !");

                    let old_pos = failover_position.get(failover_ip).unwrap_or(&"".to_string()).to_string();
                    let new_pos = lease.spec.unwrap().holder_identity.unwrap().to_string();
                    if new_pos != old_pos {
                        info!("Failover happening: {} is going to {} !", failover_ip, new_pos);
                        oneprovider.failover(failover_ip, new_pos.as_str()).await?;
                        failover_position.insert(failover_ip.to_string(), new_pos.to_string());
                        info!("Failover done")
                    }
                }
            },
            Watched::Node(node) => debug!("Got Node: {}", node.name_any()),
        }
    }

    Ok(())
}
