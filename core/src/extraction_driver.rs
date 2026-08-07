use qdrant_client::Qdrant;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};

pub async fn sinc_soberania(client: &Qdrant, id: u64, vec_768: Vec<f32>, meta: String) -> anyhow::Result<()> {
    let payload_value = serde_json::json!({ "origen": "Ruta Maestra", "data": meta });
    let point = PointStruct::new(id.to_string(), vec_768, qdrant_client::Payload::try_from(payload_value).unwrap());
    client.upsert_points(UpsertPointsBuilder::new("nexus_vectors", vec![point])).await?;
    Ok(())
}
